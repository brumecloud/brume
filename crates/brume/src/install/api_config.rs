//! `/etc/brume/api.toml`: how the panel is reached.

use std::path::Path;

use brume_api::{ApiFileConfig, DEFAULT_PORT, SYSTEM_CONFIG_PATH};

use super::Step;
use super::host::{Cmd, Host};
use super::systemd::{self, Service};

/// Writes `api.toml`, and restarts the API if it runs with another one.
pub struct ApiConfig {
    pub file: ApiFileConfig,
}

impl ApiConfig {
    /// Through an SSH tunnel: on 127.0.0.1, without tailnet names or logins.
    pub fn ssh_tunnel() -> Self {
        Self {
            file: ApiFileConfig {
                port: DEFAULT_PORT,
                hosts: Vec::new(),
                tailscale_logins: Vec::new(),
                public_url: String::new(),
            },
        }
    }

    fn text(&self) -> anyhow::Result<String> {
        Ok(format!(
            "# How the panel is reached. Written by `brume install`: the next run overwrites any \
             change.\n{}",
            toml::to_string(&self.file)?
        ))
    }
}

impl Step for ApiConfig {
    fn name(&self) -> &str {
        "Write /etc/brume/api.toml"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        host.has_file(Path::new(SYSTEM_CONFIG_PATH), &self.text()?)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let text = self.text()?;
        // What `brume api --system` would refuse is never written.
        ApiFileConfig::parse(&text)?;
        host.write(Path::new(SYSTEM_CONFIG_PATH), &text, 0o644)?;
        let unit = Service::Api.unit();
        if systemd::is_active(host, &unit)? {
            host.change(&Cmd::new("systemctl", &["restart", &unit]))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use brume_api::ApiFileConfig;

    use super::ApiConfig;

    #[test]
    fn an_ssh_tunnel_reaches_the_panel_on_localhost() {
        let text = ApiConfig::ssh_tunnel().text().expect("serializes");
        let file = ApiFileConfig::parse(&text).expect("parses");
        assert_eq!(file.panel_url(), "http://127.0.0.1:7878");
        assert!(file.hosts.is_empty() && file.tailscale_logins.is_empty());
    }
}
