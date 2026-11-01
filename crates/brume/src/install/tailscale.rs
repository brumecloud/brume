//! Tailscale: from its own apt repository, the server joins the admin's tailnet without an auth
//! key, and `tailscale serve` puts the panel on the tailnet over HTTPS.
//!
//! The admin clicks a link that the install prints: `tailscale up` gives it, and the install
//! waits until the server is on the tailnet. The panel's allowlist is the login of the person
//! who clicked, the node's owner, read from `tailscale status --json`.

use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context as _, bail};
use brume_api::{ApiFileConfig, DEFAULT_PORT};
use serde::Deserialize;

use super::host::{Cmd, Host, Output};
use super::nftables::apt_get;
use super::preflight::Platform;
use super::wait::{Console, Poll, Prompt, link, wait_for};
use super::{Step, docker, systemd};

/// The key that signs Tailscale's repository, "Tailscale Inc. (Package repository signing
/// key) <info@tailscale.com>", fingerprint 2596 A99E AAB3 3821 893C 0A79 458C A832 957F 5868.
/// The same key signs the Ubuntu and the Debian repositories.
const KEY: &str = include_str!("tailscale.asc");

const KEY_PATH: &str = "/etc/apt/keyrings/tailscale.asc";
const SOURCES_PATH: &str = "/etc/apt/sources.list.d/tailscale.sources";

const PACKAGE: &str = "tailscale";
const UNIT: &str = "tailscaled.service";

/// The server's name on the tailnet: `brume.<tailnet>.ts.net`, or `brume-1…` if it is taken.
pub const NODE_NAME: &str = "brume";

/// How long `tailscale up` waits for the login before it hands over to the install, which then
/// asks `tailscale status`.
const UP_TIMEOUT: &str = "--timeout=10s";

/// How long `tailscale serve` may wait for HTTPS before the install takes its link and waits
/// itself, then how long the actual serve may take, in seconds.
const SERVE_PROBE_SECONDS: &str = "15";
const SERVE_SECONDS: &str = "60";

/// The exit code of `timeout` when it stops the command.
const TIMED_OUT: i32 = 124;

/// The backend state of a node on the tailnet.
pub const RUNNING: &str = "Running";

/// Where the admin approves a new device, if the tailnet requires it.
const MACHINES_URL: &str = "https://login.tailscale.com/admin/machines";

/// Where the admin turns HTTPS on, if `tailscale serve` gives no link.
const DNS_URL: &str = "https://login.tailscale.com/admin/dns";

/// The user that owns the tagged devices of a tailnet.
const TAGGED_DEVICES: &str = "tagged-devices";

/// `tailscale status --json`, the fields Brume reads.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    #[serde(default)]
    backend_state: String,
    /// The login link, while the node needs a login.
    #[serde(rename = "AuthURL", default)]
    auth_url: String,
    #[serde(rename = "Self")]
    node: Option<Node>,
    /// The users of the node and its peers, by id.
    #[serde(default)]
    user: Option<BTreeMap<String, UserProfile>>,
    /// The names the node gets HTTPS certificates for: none while the tailnet has HTTPS off.
    #[serde(default)]
    cert_domains: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Node {
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "UserID", default)]
    user_id: i64,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    capabilities: Option<Vec<String>>,
    #[serde(default)]
    cap_map: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserProfile {
    #[serde(default)]
    login_name: String,
}

impl Status {
    pub fn parse(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).context("`tailscale status --json` printed an unexpected answer")
    }

    /// `Running` once the node is on the tailnet, `NeedsLogin` before.
    pub fn backend_state(&self) -> &str {
        &self.backend_state
    }

    fn running(&self) -> bool {
        self.backend_state == RUNNING
    }

    /// Whether the tailnet gives this node HTTPS certificates, which `tailscale serve` needs.
    fn https(&self) -> bool {
        let capability = self.node.as_ref().is_some_and(|node| {
            node.capabilities
                .iter()
                .flatten()
                .any(|capability| capability == "https")
                || node
                    .cap_map
                    .as_ref()
                    .is_some_and(|map| map.contains_key("https"))
        });
        capability
            || self
                .cert_domains
                .as_ref()
                .is_some_and(|names| !names.is_empty())
    }

    /// The node's name on the tailnet, such as `brume.tail1234.ts.net`.
    pub fn dns_name(&self) -> anyhow::Result<String> {
        let name = self
            .node
            .as_ref()
            .map(|node| node.dns_name.trim_end_matches('.').to_ascii_lowercase())
            .unwrap_or_default();
        if name.is_empty() {
            bail!(
                "Tailscale gives this server no name on the tailnet. Turn MagicDNS on in the \
                 Tailscale admin console ({DNS_URL}), then run the same command again."
            );
        }
        Ok(name)
    }

    /// The login of the person who owns the node: the panel's allowlist. A tagged node has no
    /// such person.
    pub fn owner_login(&self) -> anyhow::Result<String> {
        let node = self
            .node
            .as_ref()
            .context("`tailscale status` shows no node")?;
        let login = self
            .user
            .as_ref()
            .and_then(|users| users.get(&node.user_id.to_string()))
            .map(|user| user.login_name.as_str())
            .unwrap_or_default();
        let tagged = node.tags.as_ref().is_some_and(|tags| !tags.is_empty());
        if tagged || login.is_empty() || login == TAGGED_DEVICES {
            bail!(
                "This server joined the tailnet as a tagged device, which belongs to no person: \
                 the panel lets in the login of the server's owner only. Remove its tags in the \
                 Tailscale admin console ({MACHINES_URL}), or log it in again as yourself with \
                 `sudo tailscale up --ssh --hostname={NODE_NAME} --force-reauth`, then run the \
                 same command again."
            );
        }
        Ok(login.to_owned())
    }

    /// `/etc/brume/api.toml` for a panel behind `tailscale serve`: the node's name, its
    /// owner's login, and the HTTPS URL.
    pub fn panel_file(&self) -> anyhow::Result<ApiFileConfig> {
        let name = self.dns_name()?;
        Ok(ApiFileConfig {
            port: DEFAULT_PORT,
            public_url: format!("https://{name}"),
            hosts: vec![name],
            tailscale_logins: vec![self.owner_login()?],
        })
    }
}

/// `tailscale status --json`.
pub fn status_query() -> Cmd {
    Cmd::new("tailscale", &["status", "--json"])
}

/// The node's status, or `None` if Tailscale does not answer: not installed, or its daemon
/// does not run.
pub fn status(host: &Host) -> anyhow::Result<Option<Status>> {
    let output = host.query(&status_query())?;
    if !output.success() {
        return Ok(None);
    }
    Status::parse(&output.stdout).map(Some)
}

/// The running node's status, if Tailscale is up.
fn running(host: &Host) -> anyhow::Result<Option<Status>> {
    Ok(status(host)?.filter(Status::running))
}

/// The link that `tailscale up` or `tailscale serve` printed for the admin: `AuthURL` in the
/// JSON of `tailscale up --json`, else the first link after "visit:", as both print it. The
/// warnings around it have links too, to Tailscale's docs.
pub fn printed_link(output: &Output) -> Option<String> {
    let from_json = serde_json::Deserializer::from_str(&output.stdout)
        .into_iter::<serde_json::Value>()
        .map_while(Result::ok)
        .find_map(|value| value.get("AuthURL")?.as_str().and_then(link));
    from_json.or_else(|| {
        [&output.stdout, &output.stderr]
            .into_iter()
            .find_map(|text| {
                let (_, after) = text.split_once("visit:")?;
                after.split_whitespace().find_map(link)
            })
    })
}

/// The apt source of Tailscale's repository, signed by `KEY`.
fn sources(platform: &Platform) -> String {
    format!(
        "# Tailscale's apt repository, added by `brume install`.\nTypes: deb\nURIs: \
         https://pkgs.tailscale.com/stable/{distro}\nSuites: {codename}\nComponents: main\n\
         Signed-By: {KEY_PATH}\n",
        distro = platform.distro.name(),
        codename = platform.codename,
    )
}

/// Adds Tailscale's apt repository, with its key.
pub struct Repository {
    pub platform: Platform,
}

impl Step for Repository {
    fn name(&self) -> &str {
        "Add Tailscale's apt repository"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(host.has_file(Path::new(KEY_PATH), KEY)?
            && host.has_file(Path::new(SOURCES_PATH), &sources(&self.platform))?)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.write(Path::new(KEY_PATH), KEY, 0o644)?;
        host.write(Path::new(SOURCES_PATH), &sources(&self.platform), 0o644)
    }
}

/// Installs Tailscale.
pub struct Package;

impl Step for Package {
    fn name(&self) -> &str {
        "Install Tailscale from pkgs.tailscale.com"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(docker::installed_version(host, PACKAGE)?.is_some())
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.change(&apt_get(&["update"]))?;
        host.change(&apt_get(&["install", "--no-install-recommends", PACKAGE]))?;
        Ok(())
    }
}

/// Starts `tailscaled`, now and at each boot.
pub struct Daemon;

impl Step for Daemon {
    fn name(&self) -> &str {
        "Start Tailscale"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(systemd::is_enabled(host, UNIT)? && systemd::is_active(host, UNIT)?)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.change(&Cmd::new("systemctl", &["enable", "--now", UNIT]))?;
        Ok(())
    }
}

/// `tailscale up`, with Tailscale SSH, without an auth key: it hands over after
/// `UP_TIMEOUT`, with the login link in its JSON.
pub fn up_command() -> Cmd {
    Cmd::new(
        "tailscale",
        &[
            "up",
            "--ssh",
            &format!("--hostname={NODE_NAME}"),
            "--json",
            UP_TIMEOUT,
        ],
    )
}

/// Joins the admin's tailnet: prints the login link and waits for the admin to follow it.
pub struct Up {
    pub console: Rc<Console>,
}

impl Step for Up {
    fn name(&self) -> &str {
        "Join your tailnet: open the link printed here"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(running(host)?.is_some_and(|status| status.owner_login().is_ok()))
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let status = match running(host)? {
            Some(status) => status,
            None => {
                let mut output = host.attempt(&up_command())?;
                let mut link = printed_link(&output);
                if link.is_none() && auth_url(host)?.is_none() {
                    // Tailscale had no link yet when `tailscale up` handed over: once more.
                    output = host.attempt(&up_command())?;
                    link = printed_link(&output);
                }
                wait_for(&self.console, "the Tailscale login", || {
                    let status = status(host)?;
                    let state = status
                        .as_ref()
                        .map(|status| status.backend_state.clone())
                        .unwrap_or_default();
                    match (status, state.as_str()) {
                        (Some(status), RUNNING) => Ok(Poll::Ready(status)),
                        (_, "NeedsMachineAuth") => Ok(Poll::Waiting(Some(Prompt {
                            action: "Logged in. Your tailnet requires approving new devices: \
                                     approve the machine named brume in the admin console:",
                            link: MACHINES_URL.to_owned(),
                        }))),
                        (Some(status), "NeedsLogin" | "Starting" | "NoState") => {
                            if let Some(url) = super::wait::link(&status.auth_url) {
                                link = Some(url);
                            }
                            Ok(Poll::Waiting(link.clone().map(|link| Prompt {
                                action: "To put this server on your tailnet, open this link \
                                         and log in to Tailscale:",
                                link,
                            })))
                        }
                        // Tailscale stopped, or does not answer: `tailscale up` said why.
                        _ if !output.success() => Err(host_failure(&output)),
                        _ => Ok(Poll::Waiting(None)),
                    }
                })?
            }
        };
        // A tagged node is refused here, before the panel is served.
        status.owner_login()?;
        Ok(())
    }
}

/// The login link that Tailscale holds, while the node needs a login.
fn auth_url(host: &Host) -> anyhow::Result<Option<String>> {
    Ok(status(host)?.and_then(|status| link(&status.auth_url)))
}

/// The error of a failed `tailscale up`, with what it printed.
fn host_failure(output: &Output) -> anyhow::Error {
    let printed = format!("{}{}", output.stdout, output.stderr);
    anyhow::anyhow!(
        "`tailscale up` failed, and Tailscale is not up:\n{}",
        printed.trim()
    )
}

/// `tailscale serve` of the panel in the background, persistent, over HTTPS on port 443, for
/// `seconds` at most: it waits forever for HTTPS when the tailnet has it off.
pub fn serve_command(seconds: &'static str) -> Cmd {
    Cmd::new(
        "timeout",
        &[
            seconds,
            "tailscale",
            "serve",
            "--bg",
            "--https=443",
            &panel_target(),
        ],
    )
}

/// `tailscale serve status --json`.
pub fn serve_status_query() -> Cmd {
    Cmd::new("tailscale", &["serve", "status", "--json"])
}

/// Where `tailscale serve` sends the panel's requests.
fn panel_target() -> String {
    format!("http://127.0.0.1:{DEFAULT_PORT}")
}

/// `tailscale serve status --json`, the fields Brume reads.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ServeConfig {
    #[serde(rename = "TCP", default)]
    tcp: Option<BTreeMap<String, TcpHandler>>,
    #[serde(default)]
    web: Option<BTreeMap<String, WebServer>>,
}

#[derive(Debug, Default, Deserialize)]
struct TcpHandler {
    #[serde(rename = "HTTPS", default)]
    https: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WebServer {
    #[serde(default)]
    handlers: Option<BTreeMap<String, Handler>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Handler {
    #[serde(default)]
    proxy: String,
}

impl ServeConfig {
    /// Whether HTTPS on port 443 of `name` serves the panel.
    fn serves_panel(&self, name: &str) -> bool {
        let https = self
            .tcp
            .as_ref()
            .and_then(|tcp| tcp.get("443"))
            .is_some_and(|handler| handler.https);
        let proxy = self
            .web
            .as_ref()
            .and_then(|web| web.get(&format!("{name}:443")))
            .and_then(|server| server.handlers.as_ref()?.get("/"))
            .map(|handler| handler.proxy.as_str());
        https && proxy == Some(panel_target().as_str())
    }
}

/// Serves the panel on the tailnet, over HTTPS. If the tailnet has HTTPS off, prints
/// Tailscale's link that turns it on, and waits.
pub struct Serve {
    pub console: Rc<Console>,
}

impl Serve {
    fn serving(host: &Host, status: &Status) -> anyhow::Result<bool> {
        let name = status.dns_name()?;
        let Some(json) = host.query_stdout(&serve_status_query())? else {
            return Ok(false);
        };
        // Nothing served yet prints nothing, or `{}`.
        let config: ServeConfig = serde_json::from_str(&json).unwrap_or_default();
        Ok(config.serves_panel(&name))
    }
}

impl Step for Serve {
    fn name(&self) -> &str {
        "Serve the panel on your tailnet over HTTPS (tailscale serve)"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        match running(host)? {
            Some(status) => Ok(status.dns_name().is_ok() && Self::serving(host, &status)?),
            None => Ok(false),
        }
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let status = running(host)?.context("Tailscale is not up: the panel cannot be served.")?;
        status.dns_name()?;
        if !status.https() {
            // `tailscale serve` prints the link that turns HTTPS on, then waits for it.
            let output = host.attempt(&serve_command(SERVE_PROBE_SECONDS))?;
            if !output.success() {
                let link = match printed_link(&output) {
                    Some(link) => link,
                    // Still waiting when `timeout` stopped it, without a link.
                    None if output.code == Some(TIMED_OUT) => DNS_URL.to_owned(),
                    None => bail!(
                        "`tailscale serve` failed:\n{}",
                        format!("{}{}", output.stdout, output.stderr).trim()
                    ),
                };
                let prompt = Prompt {
                    action: "Your tailnet has HTTPS turned off, and the panel needs it. To turn \
                             it on, open:",
                    link,
                };
                wait_for(&self.console, "HTTPS on your tailnet", || {
                    Ok(match running(host)? {
                        Some(status) if status.https() => Poll::Ready(()),
                        _ => Poll::Waiting(Some(prompt.clone())),
                    })
                })?;
            }
        }
        host.change(&serve_command(SERVE_SECONDS))?;
        let status = running(host)?.context("Tailscale stopped.")?;
        if !Self::serving(host, &status)? {
            bail!(
                "`tailscale serve` ran, and the panel is still not served on {}. Check `tailscale \
                 serve status`.",
                status.dns_name()?
            );
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod fake {
    //! What `tailscale` answers on a host, in the tests.

    /// `tailscale status --json` of a node in `state`, owned by `login`, maybe tagged, with
    /// HTTPS on or off.
    pub fn status_json(state: &str, login: &str, tags: &[&str], https: bool) -> String {
        let cert_domains = if https {
            "[\"brume.tail1234.ts.net\"]"
        } else {
            "null"
        };
        format!(
            r#"{{
  "Version": "1.90.6-t8a5c4c3f4-g5b0f6e1a2",
  "TUN": true,
  "BackendState": "{state}",
  "AuthURL": "{auth_url}",
  "TailscaleIPs": ["100.101.102.103", "fd7a:115c:a1e0::1"],
  "Self": {{
    "ID": "nTSv3h5CNTRL",
    "PublicKey": "nodekey:5d2c1b0a",
    "HostName": "brume",
    "DNSName": "brume.tail1234.ts.net.",
    "OS": "linux",
    "UserID": 6789,
    "Tags": {tags},
    "Online": true,
    "Capabilities": ["https://tailscale.com/cap/is-admin", "ssh"],
    "CapMap": {{"ssh": null}}
  }},
  "Health": [],
  "MagicDNSSuffix": "tail1234.ts.net",
  "CurrentTailnet": {{"Name": "paul@example.com", "MagicDNSSuffix": "tail1234.ts.net", "MagicDNSEnabled": true}},
  "CertDomains": {cert_domains},
  "Peer": {{}},
  "User": {{
    "6789": {{"ID": 6789, "LoginName": "{login}", "DisplayName": "Paul", "ProfilePicURL": ""}},
    "2147455555": {{"ID": 2147455555, "LoginName": "tagged-devices", "DisplayName": "tagged-devices", "ProfilePicURL": ""}}
  }}
}}"#,
            auth_url = if state == "NeedsLogin" {
                "https://login.tailscale.com/a/1a2b3c4d5e6f"
            } else {
                ""
            },
            tags = if tags.is_empty() {
                "null".to_owned()
            } else {
                serde_json::to_string(tags).expect("serializes")
            },
        )
    }

    /// `tailscale serve status --json` once the panel is served.
    pub const SERVING: &str = r#"{
  "TCP": {"443": {"HTTPS": true}},
  "Web": {"brume.tail1234.ts.net:443": {"Handlers": {"/": {"Proxy": "http://127.0.0.1:7878"}}}}
}"#;
}

#[cfg(test)]
mod tests {
    use super::fake::{SERVING, status_json};
    use super::{ServeConfig, Status, printed_link, sources};
    use crate::install::host::Output;
    use crate::install::preflight::{Distro, Platform};

    #[test]
    fn parses_login_url_from_tailscale_up() {
        // `tailscale up --json`, with a warning that has a link of its own.
        let up = Output {
            code: Some(1),
            stdout: "{\n\t\"AuthURL\": \"https://login.tailscale.com/a/1a2b3c4d5e6f\",\n\t\"QR\": \
                     \"data:image/png;base64,iVBORw0KGgo=\",\n\t\"BackendState\": \
                     \"NeedsLogin\"\n}\n"
                .to_owned(),
            stderr: "Warning: UDP GRO forwarding is suboptimally configured on eth0, UDP \
                     forwarding throughput capability will increase with a configuration \
                     change.\nSee https://tailscale.com/s/ethtool-config-udp-gro\ntimeout \
                     waiting for Tailscale service to enter a Running state; check health \
                     with \"tailscale status\"\n"
                .to_owned(),
        };
        assert_eq!(
            printed_link(&up).as_deref(),
            Some("https://login.tailscale.com/a/1a2b3c4d5e6f")
        );
        // Without --json, as older versions print it.
        let text = Output {
            code: Some(1),
            stdout: String::new(),
            stderr: "See https://tailscale.com/s/ethtool-config-udp-gro\n\nTo authenticate, \
                     visit:\n\n\thttps://login.tailscale.com/a/abcdef\n\n"
                .to_owned(),
        };
        assert_eq!(
            printed_link(&text).as_deref(),
            Some("https://login.tailscale.com/a/abcdef")
        );
        // `tailscale serve` on a tailnet with HTTPS off.
        let serve = Output {
            code: Some(124),
            stdout: "\nServe is not enabled on your tailnet.\nTo enable, visit:\n\n         \
                     https://login.tailscale.com/f/serve?node=nTSv3h5CNTRL\n\n"
                .to_owned(),
            stderr: String::new(),
        };
        assert_eq!(
            printed_link(&serve).as_deref(),
            Some("https://login.tailscale.com/f/serve?node=nTSv3h5CNTRL")
        );
        let nothing = Output::ok("Success.\n");
        assert_eq!(printed_link(&nothing), None);
    }

    #[test]
    fn reads_dns_name_and_owner_login_from_status_json() {
        let status =
            Status::parse(&status_json("Running", "paul@example.com", &[], true)).expect("parses");
        assert!(status.running());
        assert!(status.https());
        assert_eq!(status.dns_name().expect("a name"), "brume.tail1234.ts.net");
        assert_eq!(status.owner_login().expect("an owner"), "paul@example.com");
        let file = status.panel_file().expect("a file");
        assert_eq!(file.hosts, ["brume.tail1234.ts.net"]);
        assert_eq!(file.tailscale_logins, ["paul@example.com"]);
        assert_eq!(file.public_url, "https://brume.tail1234.ts.net");
        // What `brume api --system` reads back.
        let text = toml::to_string(&file).expect("serializes");
        assert_eq!(brume_api::ApiFileConfig::parse(&text).expect("valid"), file);

        let https_off =
            Status::parse(&status_json("Running", "paul@example.com", &[], false)).expect("parses");
        assert!(!https_off.https());
    }

    #[test]
    fn a_tagged_node_has_no_owner_for_the_panel() {
        let tagged = Status::parse(&status_json(
            "Running",
            "paul@example.com",
            &["tag:server"],
            true,
        ))
        .expect("parses");
        let error = tagged.owner_login().expect_err("tagged");
        assert!(error.to_string().contains("tagged device"), "{error}");
        let json = status_json("Running", "paul@example.com", &[], true)
            .replace("\"UserID\": 6789", "\"UserID\": 2147455555");
        let error = Status::parse(&json)
            .expect("parses")
            .owner_login()
            .expect_err("tagged-devices");
        assert!(error.to_string().contains("tagged device"), "{error}");
    }

    #[test]
    fn a_logged_out_node_has_no_name_yet() {
        let json =
            status_json("NeedsLogin", "", &[], false).replace("\"brume.tail1234.ts.net.\"", "\"\"");
        let status = Status::parse(&json).expect("parses");
        assert!(!status.running());
        assert_eq!(
            status.auth_url,
            "https://login.tailscale.com/a/1a2b3c4d5e6f"
        );
        assert!(status.dns_name().is_err());
    }

    #[test]
    fn reads_what_tailscale_serve_serves() {
        let config: ServeConfig = serde_json::from_str(SERVING).expect("parses");
        assert!(config.serves_panel("brume.tail1234.ts.net"));
        assert!(!config.serves_panel("brume-1.tail1234.ts.net"));
        let other: ServeConfig =
            serde_json::from_str(&SERVING.replace("7878", "8080")).expect("parses");
        assert!(!other.serves_panel("brume.tail1234.ts.net"));
        let empty: ServeConfig = serde_json::from_str("{}").expect("parses");
        assert!(!empty.serves_panel("brume.tail1234.ts.net"));
    }

    #[test]
    fn the_repository_matches_the_distribution() {
        let platform = Platform {
            distro: Distro::Debian,
            codename: "bookworm".to_owned(),
            arch: "arm64",
        };
        assert_eq!(
            sources(&platform),
            "# Tailscale's apt repository, added by `brume install`.\nTypes: deb\nURIs: \
             https://pkgs.tailscale.com/stable/debian\nSuites: bookworm\nComponents: main\n\
             Signed-By: /etc/apt/keyrings/tailscale.asc\n"
        );
        assert!(super::KEY.starts_with("-----BEGIN PGP PUBLIC KEY BLOCK-----\n"));
    }
}
