use std::{collections::HashSet, env, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result, anyhow};

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub public_url: String,
    pub database_url: String,
    pub storage: StorageConfig,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_allowed_ids: HashSet<i64>,
}

#[derive(Clone)]
pub enum StorageConfig {
    Filesystem {
        path: PathBuf,
    },
    S3 {
        endpoint: String,
        region: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        force_path_style: bool,
    },
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bind = optional("BRUME_BIND")
            .unwrap_or_else(|| {
                format!(
                    "[::]:{}",
                    optional("PORT").unwrap_or_else(|| "8080".to_owned())
                )
            })
            .parse()
            .context("parsing BRUME_BIND")?;
        let public_url = env::var("BRUME_PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_owned())
            .trim_end_matches('/')
            .to_owned();
        let database_url = first_required(&["BRUME_DATABASE_URL", "DATABASE_URL"])?;
        let github_allowed_ids = env::var("BRUME_GITHUB_ALLOWED_IDS")
            .unwrap_or_default()
            .split(',')
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                value
                    .trim()
                    .parse::<i64>()
                    .context("parsing GitHub allowlist ID")
            })
            .collect::<Result<HashSet<_>>>()?;
        let storage = match env::var("BRUME_STORAGE_BACKEND")
            .unwrap_or_else(|_| "filesystem".to_owned())
            .as_str()
        {
            "filesystem" => StorageConfig::Filesystem {
                path: env::var("BRUME_STORAGE_PATH")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from(".brume/storage")),
            },
            "s3" => StorageConfig::S3 {
                endpoint: first_required(&["AWS_ENDPOINT_URL", "ENDPOINT"])?,
                region: first_optional(&["AWS_REGION", "AWS_DEFAULT_REGION", "REGION"])
                    .unwrap_or_else(|| "auto".to_owned()),
                bucket: first_required(&["AWS_S3_BUCKET_NAME", "BUCKET"])?,
                access_key_id: first_required(&["AWS_ACCESS_KEY_ID", "ACCESS_KEY_ID"])?,
                secret_access_key: first_required(&["AWS_SECRET_ACCESS_KEY", "SECRET_ACCESS_KEY"])?,
                force_path_style: first_optional(&["BRUME_S3_URL_STYLE", "AWS_S3_URL_STYLE"])
                    .is_some_and(|value| value.eq_ignore_ascii_case("path")),
            },
            value => return Err(anyhow!("unsupported BRUME_STORAGE_BACKEND `{value}`")),
        };
        Ok(Self {
            bind,
            public_url,
            database_url,
            storage,
            github_client_id: required("BRUME_GITHUB_CLIENT_ID")?,
            github_client_secret: required("BRUME_GITHUB_CLIENT_SECRET")?,
            github_allowed_ids,
        })
    }
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn first_optional(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| optional(name))
}

fn first_required(names: &[&str]) -> Result<String> {
    first_optional(names).ok_or_else(|| anyhow!("one of {} is required", names.join(", ")))
}
