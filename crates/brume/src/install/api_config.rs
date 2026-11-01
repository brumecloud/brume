//! `/etc/brume/api.toml`: how the panel is reached.

use std::path::Path;

use anyhow::Context as _;
use brume_api::{ApiFileConfig, DEFAULT_PORT, SYSTEM_CONFIG_PATH};

use super::host::{Cmd, Host};
use super::systemd::{self, Service};
use super::{Step, tailscale};

/// Writes `api.toml`, and restarts the API if it runs with another one.
pub enum ApiConfig {
    /// Through an SSH tunnel: on 127.0.0.1, without tailnet names or logins.
    SshTunnel,
    /// Through `tailscale serve`: the node's name and its owner's login, which exist once the
    /// node is on the tailnet, and only then.
    Tailscale,
}

impl ApiConfig {
    /// The file, or `None` while the node is not on the tailnet.
    fn file(&self, host: &Host) -> anyhow::Result<Option<ApiFileConfig>> {
        match self {
            Self::SshTunnel => Ok(Some(ApiFileConfig {
                port: DEFAULT_PORT,
                hosts: Vec::new(),
                tailscale_logins: Vec::new(),
                public_url: String::new(),
            })),
            Self::Tailscale => match tailscale::status(host)? {
                Some(status) if status.backend_state() == tailscale::RUNNING => {
                    status.panel_file().map(Some)
                }
                _ => Ok(None),
            },
        }
    }
}

fn text(file: &ApiFileConfig) -> anyhow::Result<String> {
    Ok(format!(
        "# How the panel is reached. Written by `brume install`: the next run overwrites any \
         change.\n{}",
        toml::to_string(file)?
    ))
}

impl Step for ApiConfig {
    fn name(&self) -> &str {
        "Write /etc/brume/api.toml"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        // A node that is not on the tailnet, or tagged, is for `apply` to explain.
        match self.file(host) {
            Ok(Some(file)) => host.has_file(Path::new(SYSTEM_CONFIG_PATH), &text(&file)?),
            Ok(None) | Err(_) => Ok(false),
        }
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let file = self
            .file(host)?
            .context("Tailscale is not up: the panel's name on the tailnet is unknown.")?;
        let text = text(&file)?;
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

    use super::{ApiConfig, text};
    use crate::install::host::fake::FakeSystem;
    use crate::install::host::{Host, Output};
    use crate::install::tailscale::fake::status_json;
    use crate::install::tailscale::{self};

    #[test]
    fn an_ssh_tunnel_reaches_the_panel_on_localhost() {
        let host = Host::new(Box::new(FakeSystem::default()));
        let file = ApiConfig::SshTunnel
            .file(&host)
            .expect("reads")
            .expect("a file");
        let file = ApiFileConfig::parse(&text(&file).expect("serializes")).expect("parses");
        assert_eq!(file.panel_url(), "http://127.0.0.1:7878");
        assert!(file.hosts.is_empty() && file.tailscale_logins.is_empty());
    }

    #[test]
    fn the_tailnet_name_and_login_come_from_tailscale() {
        let fake = FakeSystem::default();
        let host = Host::new(Box::new(fake.clone()));
        // Not on the tailnet yet: nothing to write.
        assert_eq!(ApiConfig::Tailscale.file(&host).expect("reads"), None);
        fake.answer(
            &tailscale::status_query(),
            Output::ok(&status_json("Running", "paul@example.com", &[], true)),
        );
        let file = ApiConfig::Tailscale
            .file(&host)
            .expect("reads")
            .expect("a file");
        let text = text(&file).expect("serializes");
        assert_eq!(
            text,
            "# How the panel is reached. Written by `brume install`: the next run overwrites any \
             change.\nport = 7878\nhosts = [\"brume.tail1234.ts.net\"]\ntailscale_logins = \
             [\"paul@example.com\"]\npublic_url = \"https://brume.tail1234.ts.net\"\n"
        );
        assert_eq!(ApiFileConfig::parse(&text).expect("valid"), file);
    }
}
