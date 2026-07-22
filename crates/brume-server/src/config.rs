use std::{collections::HashSet, env, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result, anyhow};

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    #[allow(dead_code)]
    pub api_public_url: String,
    pub api_public_host: String,
    pub auth_public_url: String,
    pub auth_public_host: String,
    pub plan_public_url: String,
    pub plan_public_host: String,
    pub public_domain: String,
    pub public_scheme: String,
    pub public_port: Option<u16>,
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
        let (api_public_url, api_public_host, _) =
            public_origin("BRUME_API_PUBLIC_URL", "http://localhost:8080")?;
        let (auth_public_url, auth_public_host, _) =
            public_origin("BRUME_AUTH_PUBLIC_URL", &api_public_url)?;
        let (plan_public_url, plan_public_host, parsed_plan_url) =
            public_origin("BRUME_PLAN_PUBLIC_URL", &api_public_url)?;
        let public_domain = required("BRUME_PUBLIC_DOMAIN")?.to_ascii_lowercase();
        validate_public_domain(&public_domain)?;
        let public_scheme = parsed_plan_url.scheme().to_owned();
        let public_port = parsed_plan_url.port();
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
            api_public_url,
            api_public_host,
            auth_public_url,
            auth_public_host,
            plan_public_url,
            plan_public_host,
            public_domain,
            public_scheme,
            public_port,
            database_url,
            storage,
            github_client_id: required("BRUME_GITHUB_CLIENT_ID")?,
            github_client_secret: required("BRUME_GITHUB_CLIENT_SECRET")?,
            github_allowed_ids,
        })
    }
}

impl Config {
    pub fn public_url(&self, label: &str) -> String {
        let port = self
            .public_port
            .filter(|port| {
                !matches!(
                    (self.public_scheme.as_str(), *port),
                    ("http", 80) | ("https", 443)
                )
            })
            .map(|port| format!(":{port}"))
            .unwrap_or_default();
        format!(
            "{}://{}.{}{}",
            self.public_scheme, label, self.public_domain, port
        )
    }
}

fn public_origin(name: &str, default: &str) -> Result<(String, String, url::Url)> {
    let value = env::var(name)
        .unwrap_or_else(|_| default.to_owned())
        .trim_end_matches('/')
        .to_owned();
    let parsed = url::Url::parse(&value).with_context(|| format!("parsing {name}"))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(anyhow!(
            "{name} must be an http(s) origin without credentials, a path, a query, or a fragment"
        ));
    }
    let mut host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("{name} must contain a host"))?
        .to_owned();
    if let Some(port) = parsed.port()
        && !matches!((parsed.scheme(), port), ("http", 80) | ("https", 443))
    {
        host.push(':');
        host.push_str(&port.to_string());
    }
    Ok((value, host, parsed))
}

fn validate_public_domain(domain: &str) -> Result<()> {
    if domain.is_empty()
        || domain.len() > 253
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label.chars().all(|value| {
                    value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-'
                })
        })
    {
        return Err(anyhow!(
            "BRUME_PUBLIC_DOMAIN must be a valid lowercase DNS name"
        ));
    }
    Ok(())
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
