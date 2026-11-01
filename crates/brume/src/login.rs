//! `brume admin login`, run as root on a host: a one-time login link to the panel, from the
//! engine. `brume install` prints one the same way.

use std::net::IpAddr;
use std::path::Path;

use anyhow::bail;
use brume_api::{ApiFileConfig, SYSTEM_CONFIG_PATH};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_runtime::paths::Paths;

use crate::admin::admin_request;

/// A one-time login link's token, and when it expires.
pub struct LoginLink {
    token: String,
    expires_at: String,
}

/// Asks the engine of this host for a login link and prints it.
pub fn run() -> anyhow::Result<()> {
    // Before the request: a link that cannot be printed right is not created.
    let file = ApiFileConfig::load(Path::new(SYSTEM_CONFIG_PATH))?;
    let link = request_link(&Paths::system())?;
    print_link(&file, &link, &ssh_destination_from_env());
    Ok(())
}

/// Asks the engine on `paths`' `admin.sock` for a login link.
pub fn request_link(paths: &Paths) -> anyhow::Result<LoginLink> {
    match admin_request(paths, AdminRequest::CreateLoginLink)? {
        AdminReply::LoginLink { token, expires_at } => Ok(LoginLink { token, expires_at }),
        #[allow(
            unreachable_patterns,
            reason = "other requests of admin.sock get other replies"
        )]
        _ => bail!("The engine answered with something other than a login link."),
    }
}

/// Prints `link` for the panel of `file`. Without a public URL, the panel is reached through
/// an SSH tunnel to `ssh_destination`, such as `admin@203.0.113.7`: the tunnel comes first.
pub fn print_link(file: &ApiFileConfig, link: &LoginLink, ssh_destination: &str) {
    let url = login_url(file, &link.token);
    if file.public_url.is_empty() {
        println!(
            "The panel listens on 127.0.0.1:{port} of this server. From your computer:",
            port = file.port
        );
        println!(
            "  1. Open an SSH tunnel: ssh -L {port}:127.0.0.1:{port} {ssh_destination}",
            port = file.port
        );
        println!("  2. Open {url}");
    } else {
        println!("Open {url}");
    }
    println!(
        "The link works once, until {}. `sudo brume admin login` prints a new one.",
        link.expires_at
    );
}

/// The panel's login page with `token`, which the panel reads from the fragment: it never
/// reaches a server log.
fn login_url(file: &ApiFileConfig, token: &str) -> String {
    format!("{}/login#token={token}", file.panel_url())
}

/// The SSH destination of this server, as `sudo` and `sshd` describe it in the environment.
pub fn ssh_destination_from_env() -> String {
    ssh_destination(
        std::env::var("SUDO_USER").ok().as_deref(),
        std::env::var("SSH_CONNECTION").ok().as_deref(),
    )
}

/// `<user>@<address>`: the user who ran `sudo`, and the server's address in the SSH session
/// that runs the command, `SSH_CONNECTION`'s third field. A placeholder for what is unknown.
fn ssh_destination(sudo_user: Option<&str>, ssh_connection: Option<&str>) -> String {
    let user = sudo_user
        .filter(|user| {
            !user.is_empty()
                && user != &"root"
                && user
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        })
        .unwrap_or("<user>");
    let address = ssh_connection
        .and_then(|connection| connection.split_whitespace().nth(2))
        .and_then(|address| address.parse::<IpAddr>().ok())
        .map_or_else(|| "<server>".to_owned(), |address| address.to_string());
    format!("{user}@{address}")
}

#[cfg(test)]
mod tests {
    use brume_api::ApiFileConfig;

    use super::{login_url, ssh_destination};

    #[test]
    fn the_link_opens_the_public_url() {
        let file = ApiFileConfig {
            port: 7878,
            hosts: vec!["brume.tail1234.ts.net".to_owned()],
            tailscale_logins: vec!["admin@example.com".to_owned()],
            public_url: "https://brume.tail1234.ts.net/".to_owned(),
        };
        assert_eq!(
            login_url(&file, "tok"),
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
            login_url(&file, "tok"),
            "http://127.0.0.1:7979/login#token=tok"
        );
    }

    #[test]
    fn the_tunnel_goes_to_the_server_of_the_ssh_session() {
        assert_eq!(
            ssh_destination(Some("paul"), Some("198.51.100.4 52814 203.0.113.7 22")),
            "paul@203.0.113.7"
        );
        assert_eq!(
            ssh_destination(Some("admin"), Some("2001:db8::4 52814 2001:db8::7 22")),
            "admin@2001:db8::7"
        );
        assert_eq!(ssh_destination(None, None), "<user>@<server>");
        assert_eq!(
            ssh_destination(Some("root"), Some("garbage")),
            "<user>@<server>"
        );
        assert_eq!(
            ssh_destination(Some("a b"), Some("1.2.3.4 1 not-an-ip 22")),
            "<user>@<server>"
        );
    }
}
