use std::{
    collections::BTreeMap,
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use brume_api_client::BrumeClient;
use brume_core::{AuthMode, TokenPair};
use chrono::{Duration, Utc};
use directories::BaseDirs;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

const CREDENTIALS_FILE_NAME: &str = "credentials.json";
const CREDENTIALS_LOCK_FILE_NAME: &str = "credentials.lock";

#[derive(Debug, Default, Deserialize)]
pub struct ProjectFile {
    #[serde(default)]
    pub plan: PlanConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct PlanConfig {
    pub title: Option<String>,
    pub entry: Option<String>,
    pub slug: Option<String>,
    pub auth: Option<AuthMode>,
    pub overlay: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct CredentialsFile {
    #[serde(default)]
    credentials: BTreeMap<String, TokenPair>,
}

pub fn load_project(directory: &Path) -> Result<ProjectFile> {
    let path = directory.join("brume.toml");
    if !path.exists() {
        return Ok(ProjectFile::default());
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("reading project configuration {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("parsing {}", path.display()))
}

pub fn default_slug(directory: &Path) -> Result<String> {
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("cannot derive a slug from {}", directory.display()))?;
    let slug = name
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    validate_slug(&slug)?;
    Ok(slug)
}

pub fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() || slug.len() > 80 {
        bail!("a URL slug must contain between 1 and 80 characters");
    }
    if slug.starts_with('-')
        || slug.ends_with('-')
        || !slug.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        bail!("URL slugs may contain lowercase ASCII letters, digits, and internal hyphens only");
    }
    Ok(())
}

pub async fn load_access_token(base_url: &str) -> Result<String> {
    if let Ok(token) = env::var("BRUME_TOKEN")
        && !token.trim().is_empty()
    {
        return Ok(token);
    }
    let credentials_path = credentials_path()?;
    let lock = open_credentials_lock(&credentials_path)?;
    lock.lock_exclusive()
        .context("locking the Brume credentials")?;
    let mut credentials = load_credentials_from(&credentials_path, base_url)?;
    if credentials.access_expires_at <= Utc::now() + Duration::seconds(30) {
        credentials = BrumeClient::new(base_url, None)?
            .refresh_token(credentials.refresh_token)
            .await
            .context("refreshing the Brume access token")?;
        save_credentials_to(&credentials_path, base_url, &credentials)?;
    }
    FileExt::unlock(&lock).context("unlocking the Brume credentials")?;
    Ok(credentials.access_token)
}

fn load_credentials_from(path: &Path, base_url: &str) -> Result<TokenPair> {
    let file = read_credentials_file(path)?;
    file.credentials.get(base_url).cloned().ok_or_else(|| {
        anyhow!("not logged in to {base_url}; run `brume --base-url {base_url} login` first")
    })
}

pub fn save_credentials(base_url: &str, credentials: &TokenPair) -> Result<()> {
    let credentials_path = credentials_path()?;
    let lock = open_credentials_lock(&credentials_path)?;
    lock.lock_exclusive()
        .context("locking the Brume credentials")?;
    save_credentials_to(&credentials_path, base_url, credentials)?;
    FileExt::unlock(&lock).context("unlocking the Brume credentials")
}

fn credentials_path() -> Result<PathBuf> {
    let directories =
        BaseDirs::new().ok_or_else(|| anyhow!("could not locate the home directory"))?;
    Ok(directories
        .home_dir()
        .join(".brume")
        .join(CREDENTIALS_FILE_NAME))
}

fn open_credentials_lock(credentials_path: &Path) -> Result<fs::File> {
    let directory = credentials_path
        .parent()
        .ok_or_else(|| anyhow!("credentials path has no parent directory"))?;
    fs::create_dir_all(directory).context("creating the Brume credentials directory")?;
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(directory.join(CREDENTIALS_LOCK_FILE_NAME))
        .context("opening the Brume credentials lock")
}

fn read_credentials_file(path: &Path) -> Result<CredentialsFile> {
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents)
            .with_context(|| format!("parsing the Brume credentials file {}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(CredentialsFile::default())
        }
        Err(error) => Err(error)
            .with_context(|| format!("reading the Brume credentials file {}", path.display())),
    }
}

fn save_credentials_to(path: &Path, base_url: &str, credentials: &TokenPair) -> Result<()> {
    let mut file = read_credentials_file(path)?;
    file.credentials
        .insert(base_url.to_owned(), credentials.clone());
    let mut encoded = serde_json::to_vec_pretty(&file).context("encoding the Brume credentials")?;
    encoded.push(b'\n');

    let directory = path
        .parent()
        .ok_or_else(|| anyhow!("credentials path has no parent directory"))?;
    fs::create_dir_all(directory).context("creating the Brume credentials directory")?;
    let mut temporary =
        NamedTempFile::new_in(directory).context("creating the Brume credentials file")?;
    temporary
        .write_all(&encoded)
        .context("writing the Brume credentials file")?;
    temporary
        .as_file()
        .sync_all()
        .context("syncing the Brume credentials file")?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("saving the Brume credentials file {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials(access_token: &str, refresh_token: &str) -> TokenPair {
        TokenPair {
            access_token: access_token.to_owned(),
            refresh_token: refresh_token.to_owned(),
            access_expires_at: Utc::now() + Duration::hours(1),
            refresh_expires_at: Utc::now() + Duration::days(90),
        }
    }

    #[test]
    fn credentials_file_keeps_each_base_url() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(CREDENTIALS_FILE_NAME);
        let production = credentials("access_production", "refresh_production");
        let development = credentials("access_development", "refresh_development");

        save_credentials_to(&path, "https://api.brume.dev", &production).unwrap();
        save_credentials_to(&path, "http://localhost:3000", &development).unwrap();

        let loaded_production = load_credentials_from(&path, "https://api.brume.dev").unwrap();
        let loaded_development = load_credentials_from(&path, "http://localhost:3000").unwrap();
        assert_eq!(loaded_production.access_token, production.access_token);
        assert_eq!(loaded_production.refresh_token, production.refresh_token);
        assert_eq!(loaded_development.access_token, development.access_token);
        assert_eq!(loaded_development.refresh_token, development.refresh_token);

        let plaintext = fs::read_to_string(path).unwrap();
        assert!(plaintext.contains("access_production"));
        assert!(plaintext.contains("refresh_development"));
    }

    #[test]
    fn missing_base_url_reports_that_login_is_required() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(CREDENTIALS_FILE_NAME);

        let error = load_credentials_from(&path, "https://api.brume.dev").unwrap_err();

        assert_eq!(
            error.to_string(),
            "not logged in to https://api.brume.dev; run `brume --base-url https://api.brume.dev login` first"
        );
    }
}
