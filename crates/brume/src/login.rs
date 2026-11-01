//! `brume login`, run as root on a host: a one-time login link to the panel, from the engine.

use std::path::Path;

use anyhow::bail;
use brume_api::{ApiFileConfig, SYSTEM_CONFIG_PATH};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_runtime::paths::Paths;

use crate::admin::admin_request;

/// Asks the engine of this host for a login link and prints it.
pub fn run() -> anyhow::Result<()> {
    // Before the request: a link that cannot be printed right is not created.
    let file = ApiFileConfig::load(Path::new(SYSTEM_CONFIG_PATH))?;
    let reply = admin_request(&Paths::system(), AdminRequest::CreateLoginLink)?;
    let (token, expires_at) = match reply {
        AdminReply::LoginLink { token, expires_at } => (token, expires_at),
        #[allow(
            unreachable_patterns,
            reason = "other requests of admin.sock get other replies"
        )]
        _ => bail!("The engine answered with something other than a login link."),
    };
    println!("Open {}", login_link(&file, &token));
    println!("The link works once, until {expires_at}.");
    if file.public_url.is_empty() {
        println!(
            "From your computer, forward the port first: ssh -L {port}:127.0.0.1:{port} <host>",
            port = file.port
        );
    }
    Ok(())
}

/// The panel's login page with `token`, which the panel reads from the fragment: it never
/// reaches a server log.
fn login_link(file: &ApiFileConfig, token: &str) -> String {
    format!("{}/login#token={token}", file.panel_url())
}

#[cfg(test)]
mod tests {
    use brume_api::ApiFileConfig;

    use super::login_link;

    #[test]
    fn the_link_opens_the_public_url() {
        let file = ApiFileConfig {
            port: 7878,
            hosts: vec!["brume.tail1234.ts.net".to_owned()],
            tailscale_logins: vec!["admin@example.com".to_owned()],
            public_url: "https://brume.tail1234.ts.net/".to_owned(),
        };
        assert_eq!(
            login_link(&file, "tok"),
            "https://brume.tail1234.ts.net/login#token=tok"
        );
    }

    #[test]
    fn the_link_of_an_ssh_tunnel_is_on_localhost() {
        let file = ApiFileConfig {
            port: 7979,
            hosts: vec![],
            tailscale_logins: vec![],
            public_url: String::new(),
        };
        assert_eq!(
            login_link(&file, "tok"),
            "http://127.0.0.1:7979/login#token=tok"
        );
    }
}
