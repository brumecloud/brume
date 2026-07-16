use std::{env, fs, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use brume_core::Visibility;
use serde::Deserialize;

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

pub fn load_token(base_url: &str) -> Result<String> {
    if let Ok(token) = env::var("BRUME_TOKEN")
        && !token.trim().is_empty()
    {
        return Ok(token);
    }
    let entry = keyring::Entry::new("dev.brume.cli", base_url)?;
    match entry.get_password() {
        Ok(token) => Ok(token),
        Err(keyring::Error::NoEntry) => Err(anyhow!(
            "not logged in to {base_url}; run `brume --base-url {base_url} login` first"
        )),
        Err(error) => Err(error).context("loading the Brume token from the system keychain"),
    }
}

pub fn save_token(base_url: &str, token: &str) -> Result<()> {
    keyring::Entry::new("dev.brume.cli", base_url)?
        .set_password(token)
        .context("saving the Brume token in the system keychain")
}
