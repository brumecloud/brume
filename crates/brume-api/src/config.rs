//! How the API listens and whom it answers: for `brume dev`, or from `/etc/brume/api.toml` on a
//! host, where `tailscale serve` or an SSH tunnel reaches it.

use std::io::Read as _;
use std::path::Path;

use anyhow::{Context as _, bail};
use serde::{Deserialize, Serialize};

/// The port of the API and the panel on 127.0.0.1, unless told otherwise.
pub const DEFAULT_PORT: u16 = 7878;

/// The file of `brume api --system`, written by `brume install`.
pub const SYSTEM_CONFIG_PATH: &str = "/etc/brume/api.toml";

/// The port of the panel's Vite dev server, which proxies `/api` here.
const VITE_DEV_PORT: u16 = 5173;

/// The largest `api.toml`, in bytes.
const MAX_FILE_BYTES: u64 = 64 * 1024;

/// The longest host name.
const MAX_HOST_LEN: usize = 253;

/// The longest label of a host name.
const MAX_LABEL_LEN: usize = 63;

/// How the API listens, and what every request must carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiConfig {
    /// The port on 127.0.0.1.
    pub port: u16,
    /// The `Host` values accepted, such as `127.0.0.1:8080`. Any other gets a 421, which
    /// defeats DNS rebinding.
    pub allowed_hosts: Vec<String>,
    /// The `Origin` values accepted on a write, such as `http://127.0.0.1:8080`.
    pub allowed_origins: Vec<String>,
    /// If not empty, every request must carry exactly one `Tailscale-User-Login`, equal to one
    /// of these. `tailscale serve` sets it, after removing the one the client sent.
    pub tailscale_logins: Vec<String>,
    /// Whether the session cookie is `Secure`: the panel is served over HTTPS.
    pub secure_cookie: bool,
}

impl ApiConfig {
    /// `brume dev`: listens on `port`, for the panel served here and for the Vite dev server.
    pub fn new(port: u16) -> Self {
        let allowed_hosts: Vec<String> = [port, VITE_DEV_PORT]
            .into_iter()
            .flat_map(local_hosts)
            .collect();
        Self {
            port,
            allowed_origins: allowed_hosts
                .iter()
                .map(|host| format!("http://{host}"))
                .collect(),
            allowed_hosts,
            tailscale_logins: Vec::new(),
            secure_cookie: false,
        }
    }

    /// A host: `127.0.0.1` and `localhost` on the port, for an SSH tunnel, and each of `hosts`
    /// as `tailscale serve` passes it over HTTPS.
    pub fn from_file(file: &ApiFileConfig) -> Self {
        let mut allowed_hosts = local_hosts(file.port).to_vec();
        let mut allowed_origins: Vec<String> = allowed_hosts
            .iter()
            .map(|host| format!("http://{host}"))
            .collect();
        for host in &file.hosts {
            // A browser sends the origin's host in lower case.
            let host = host.to_ascii_lowercase();
            allowed_origins.push(format!("https://{host}"));
            allowed_hosts.push(format!("{host}:443"));
            allowed_hosts.push(host);
        }
        Self {
            port: file.port,
            allowed_hosts,
            allowed_origins,
            tailscale_logins: file.tailscale_logins.clone(),
            // Behind `tailscale serve`, the panel is always on HTTPS.
            secure_cookie: !file.hosts.is_empty() || file.public_url.starts_with("https://"),
        }
    }
}

/// `127.0.0.1:<port>` and `localhost:<port>`.
fn local_hosts(port: u16) -> [String; 2] {
    [format!("127.0.0.1:{port}"), format!("localhost:{port}")]
}

/// `/etc/brume/api.toml`: how a host reaches the panel. `brume install` writes it, `brume api
/// --system` and `brume admin login` read it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiFileConfig {
    /// The port on 127.0.0.1.
    #[serde(default = "default_port")]
    pub port: u16,
    /// The machine's names on the tailnet, without scheme or port, such as
    /// `brume.tail1234.ts.net`. Empty for an SSH tunnel.
    #[serde(default)]
    pub hosts: Vec<String>,
    /// The Tailscale logins that may open the panel. Required with `hosts`; empty for an SSH
    /// tunnel, which `tailscale serve` does not pass through.
    #[serde(default)]
    pub tailscale_logins: Vec<String>,
    /// The panel's URL, such as `https://brume.tail1234.ts.net`, for the links of `brume admin
    /// login`. Empty for an SSH tunnel.
    #[serde(default)]
    pub public_url: String,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

impl ApiFileConfig {
    /// Reads and checks the file at `path`.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let file =
            std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
        let mut text = String::new();
        file.take(MAX_FILE_BYTES + 1)
            .read_to_string(&mut text)
            .with_context(|| format!("cannot read {}", path.display()))?;
        if text.len() as u64 > MAX_FILE_BYTES {
            bail!("{} is larger than 64 KiB", path.display());
        }
        Self::parse(&text).with_context(|| format!("{} is invalid", path.display()))
    }

    /// Parses and checks the contents of `api.toml`.
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        let file: Self = toml::from_str(text)?;
        file.check()?;
        Ok(file)
    }

    /// The panel's URL, without a trailing slash: `public_url`, or `127.0.0.1` on the port for
    /// an SSH tunnel.
    pub fn panel_url(&self) -> String {
        match self.public_url.trim_end_matches('/') {
            "" => format!("http://127.0.0.1:{}", self.port),
            url => url.to_owned(),
        }
    }

    fn check(&self) -> anyhow::Result<()> {
        if self.port == 0 {
            bail!("`port` must not be 0");
        }
        for host in &self.hosts {
            if !is_host_name(host) {
                bail!(
                    "`hosts` holds {host:?}, which is not a host name: write the name alone, \
                     without a scheme or a port, such as brume.tail1234.ts.net"
                );
            }
        }
        for login in &self.tailscale_logins {
            if login.is_empty() || !login.bytes().all(|b| b.is_ascii_graphic()) {
                bail!("`tailscale_logins` holds {login:?}, which is not a Tailscale login");
            }
        }
        if !self.hosts.is_empty() && self.tailscale_logins.is_empty() {
            bail!(
                "`hosts` is set but `tailscale_logins` is empty: the panel would answer any \
                 device of the tailnet. Add the admin's Tailscale login."
            );
        }
        if self.hosts.is_empty() && !self.tailscale_logins.is_empty() {
            bail!(
                "`tailscale_logins` is set but `hosts` is empty: only `tailscale serve` sets the \
                 login, and it needs the machine's name on the tailnet in `hosts`."
            );
        }
        if !self.hosts.is_empty() {
            let url = self.public_url.trim_end_matches('/');
            let served = self
                .hosts
                .iter()
                .any(|host| url.strip_prefix("https://") == Some(host.as_str()));
            if !served {
                bail!(
                    "`public_url` is {:?}: with `hosts`, it must be https:// followed by one of \
                     them, such as https://{}",
                    self.public_url,
                    self.hosts[0]
                );
            }
        }
        let url = &self.public_url;
        let rest = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"));
        let valid_url = url.is_empty()
            || rest
                .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_graphic()));
        if !valid_url {
            bail!("`public_url` is {url:?}: write an http:// or https:// URL, or nothing");
        }
        Ok(())
    }
}

/// Letters, digits and hyphens in dot-separated labels.
fn is_host_name(name: &str) -> bool {
    name.len() <= MAX_HOST_LEN
        && name.split('.').all(|label| {
            (1..=MAX_LABEL_LEN).contains(&label.len())
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::{ApiConfig, ApiFileConfig};

    const TAILNET: &str = r#"
port = 7878
hosts = ["brume.tail1234.ts.net"]
tailscale_logins = ["admin@example.com"]
public_url = "https://brume.tail1234.ts.net"
"#;

    #[test]
    fn public_url_must_be_https_on_one_of_the_hosts() {
        for public_url in [
            "",
            "http://brume.tail1234.ts.net",
            "https://brume.tail1234.ts.net.",
            "https://other.tail1234.ts.net",
            "https://brume.tail1234.ts.net:8443",
        ] {
            let text = TAILNET.replace(
                "public_url = \"https://brume.tail1234.ts.net\"",
                &format!("public_url = {public_url:?}"),
            );
            assert!(ApiFileConfig::parse(&text).is_err(), "{public_url:?}");
        }
        let text = TAILNET.replace(
            "public_url = \"https://brume.tail1234.ts.net\"",
            "public_url = \"https://brume.tail1234.ts.net/\"",
        );
        assert!(ApiFileConfig::parse(&text).is_ok());
    }

    #[test]
    fn logins_without_hosts_are_refused() {
        let text = "tailscale_logins = [\"admin@example.com\"]\n";
        assert!(ApiFileConfig::parse(text).is_err());
    }

    #[test]
    fn reads_the_file_of_a_tailnet_install() {
        let file = ApiFileConfig::parse(TAILNET).expect("parses");
        assert_eq!(
            file,
            ApiFileConfig {
                port: 7878,
                hosts: vec!["brume.tail1234.ts.net".to_owned()],
                tailscale_logins: vec!["admin@example.com".to_owned()],
                public_url: "https://brume.tail1234.ts.net".to_owned(),
            }
        );
        assert_eq!(file.panel_url(), "https://brume.tail1234.ts.net");
        let reread =
            ApiFileConfig::parse(&toml::to_string(&file).expect("serializes")).expect("parses");
        assert_eq!(reread, file);
    }

    #[test]
    fn an_ssh_tunnel_needs_no_more_than_the_port() {
        let file = ApiFileConfig::parse("").expect("parses");
        assert_eq!(file.port, 7878);
        assert!(file.hosts.is_empty() && file.tailscale_logins.is_empty());
        assert_eq!(file.panel_url(), "http://127.0.0.1:7878");
        let config = ApiConfig::from_file(&file);
        assert!(config.tailscale_logins.is_empty());
        assert!(!config.secure_cookie);
        assert_eq!(config.allowed_hosts, ["127.0.0.1:7878", "localhost:7878"]);
    }

    #[test]
    fn a_tailnet_host_is_served_over_https() {
        let file =
            ApiFileConfig::parse(&TAILNET.replace("brume.tail", "Brume.tail")).expect("parses");
        let config = ApiConfig::from_file(&file);
        assert_eq!(
            config.allowed_hosts,
            [
                "127.0.0.1:7878",
                "localhost:7878",
                "brume.tail1234.ts.net:443",
                "brume.tail1234.ts.net"
            ]
        );
        assert_eq!(
            config.allowed_origins,
            [
                "http://127.0.0.1:7878",
                "http://localhost:7878",
                "https://brume.tail1234.ts.net"
            ]
        );
        assert!(config.secure_cookie);
    }

    #[test]
    fn the_link_of_a_public_url_has_no_double_slash() {
        let file =
            ApiFileConfig::parse(&TAILNET.replace(".ts.net\"\n", ".ts.net/\"\n")).expect("parses");
        assert_eq!(file.public_url, "https://brume.tail1234.ts.net/");
        assert_eq!(file.panel_url(), "https://brume.tail1234.ts.net");
    }

    #[test]
    fn refuses_a_file_that_would_open_the_panel_wrongly() {
        for (text, needle) in [
            ("tailscale_login = [\"a@b\"]", "unknown field"),
            ("port = 0", "port"),
            (
                "hosts = [\"brume.ts.net:443\"]\ntailscale_logins = [\"a@b\"]",
                "host name",
            ),
            (
                "hosts = [\"https://brume.ts.net\"]\ntailscale_logins = [\"a@b\"]",
                "host name",
            ),
            ("hosts = [\"\"]\ntailscale_logins = [\"a@b\"]", "host name"),
            ("hosts = [\"brume.ts.net\"]", "tailscale_logins"),
            ("tailscale_logins = [\"\"]", "Tailscale login"),
            ("tailscale_logins = [\"a b\"]", "Tailscale login"),
            ("public_url = \"brume.ts.net\"", "public_url"),
            ("public_url = \"https://\"", "public_url"),
        ] {
            let error = ApiFileConfig::parse(text).expect_err(text);
            assert!(format!("{error:#}").contains(needle), "{text}: {error:#}");
        }
    }

    #[test]
    fn load_names_the_file() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let path = dir.path().join("api.toml");
        let error = ApiFileConfig::load(&path).expect_err("missing");
        assert!(format!("{error:#}").contains("api.toml"), "{error:#}");
        std::fs::write(&path, "port = 0").expect("writes");
        let error = ApiFileConfig::load(&path).expect_err("invalid");
        assert!(
            format!("{error:#}").contains("api.toml is invalid"),
            "{error:#}"
        );
        std::fs::write(&path, TAILNET).expect("writes");
        assert_eq!(ApiFileConfig::load(&path).expect("loads").port, 7878);
        std::fs::write(&path, " ".repeat(65 * 1024)).expect("writes");
        let error = ApiFileConfig::load(&path).expect_err("too large");
        assert!(format!("{error:#}").contains("64 KiB"), "{error:#}");
    }
}
