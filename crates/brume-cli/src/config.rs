use std::{
    env, fs,
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use brume_api_client::BrumeClient;
use brume_core::{TokenPair, Visibility};
use chrono::{Duration, Utc};
use directories::ProjectDirs;
use fs2::FileExt;
use serde::Deserialize;
use sha2::{Digest, Sha256};

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
    pub visibility: Option<Visibility>,
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
    let lock_path = credentials_lock_path(base_url)?;
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).context("creating the Brume credentials directory")?;
    }
    let lock = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)
        .context("opening the Brume credentials lock")?;
    lock.lock_exclusive()
        .context("locking the Brume credentials")?;
    let mut credentials = load_credentials(base_url)?;
    if credentials.access_expires_at <= Utc::now() + Duration::seconds(30) {
        credentials = BrumeClient::new(base_url, None)?
            .refresh_token(credentials.refresh_token)
            .await
            .context("refreshing the Brume access token")?;
        save_credentials(base_url, &credentials)?;
    }
    lock.unlock().context("unlocking the Brume credentials")?;
    Ok(credentials.access_token)
}

fn load_credentials(base_url: &str) -> Result<TokenPair> {
    let entry = keyring::Entry::new("dev.brume.cli", base_url)?;
    match entry.get_password() {
        Ok(credentials) => serde_json::from_str(&credentials).map_err(|_| {
            anyhow!(
                "the saved login for {base_url} uses the retired token format; run `brume --base-url {base_url} login` again"
            )
        }),
        Err(keyring::Error::NoEntry) => Err(anyhow!(
            "not logged in to {base_url}; run `brume --base-url {base_url} login` first"
        )),
        Err(error) => Err(error).context("loading the Brume token from the system keychain"),
    }
}

pub fn save_credentials(base_url: &str, credentials: &TokenPair) -> Result<()> {
    let encoded = serde_json::to_string(credentials).context("encoding the Brume credentials")?;
    keyring::Entry::new("dev.brume.cli", base_url)?
        .set_password(&encoded)
        .context("saving the Brume credentials in the system keychain")
}

fn credentials_lock_path(base_url: &str) -> Result<PathBuf> {
    let directories = ProjectDirs::from("dev", "brume", "Brume")
        .ok_or_else(|| anyhow!("could not locate the Brume data directory"))?;
    let hash = hex::encode(Sha256::digest(base_url.as_bytes()));
    Ok(directories
        .data_local_dir()
        .join(format!("credentials-{hash}.lock")))
}
