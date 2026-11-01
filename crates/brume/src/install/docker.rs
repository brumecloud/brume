//! Docker, from Docker's own apt repository, configured before its first start: user
//! namespaces, no new privileges, bounded logs, Brume's address pools, no default bridge and no
//! TCP socket.

use std::path::Path;

use anyhow::bail;
use serde::Serialize;

use super::host::{Cmd, Host};
use super::nftables::apt_get;
use super::preflight::Platform;
use super::{Step, systemd};

/// Docker's configuration.
pub const DAEMON_JSON_PATH: &str = "/etc/docker/daemon.json";

/// The pool of Docker's networks, cut in `/24`. The firewall knows it: 10.213.0.0/16 holds it,
/// and the ingress network, 10.213.255.0/24, which a later version adds.
pub const ADDRESS_POOL: &str = "10.213.0.0/17";

/// The size of each network of the pool.
const NETWORK_PREFIX: u8 = 24;

/// The key that signs Docker's repository, "Docker Release (CE deb)", fingerprint
/// 9DC8 5822 9FC7 DD38 854A E2D8 8D81 803C 0EBF CD88. The same key signs the Ubuntu and the
/// Debian repositories.
const KEY: &str = include_str!("docker.asc");

const KEY_PATH: &str = "/etc/apt/keyrings/docker.asc";
const SOURCES_PATH: &str = "/etc/apt/sources.list.d/docker.sources";
const PIN_PATH: &str = "/etc/apt/preferences.d/docker";

/// The packages installed, without their recommendations.
const PACKAGES: [&str; 3] = ["docker-ce", "docker-ce-cli", "containerd.io"];

/// Packages that are a Docker of another origin.
const FOREIGN_PACKAGES: [&str; 3] = ["docker.io", "podman-docker", "moby-engine"];

/// The oldest Docker Brume runs on: Docker 28.5.2 ships runc 1.3.3, which fixes the container
/// escapes of November 2025.
const MIN_VERSION: Version = Version {
    major: 28,
    minor: 5,
    patch: 2,
};

const UNIT: &str = "docker.service";

/// `/etc/docker/daemon.json`. No `hosts`: Docker listens on its unix socket only.
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct DaemonJson {
    userns_remap: &'static str,
    no_new_privileges: bool,
    live_restore: bool,
    log_driver: &'static str,
    log_opts: LogOpts,
    default_address_pools: [AddressPool; 1],
    bridge: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct LogOpts {
    max_size: &'static str,
    max_file: &'static str,
}

#[derive(Serialize)]
struct AddressPool {
    base: &'static str,
    size: u8,
}

/// The contents of `daemon.json`.
pub fn daemon_json() -> String {
    let config = DaemonJson {
        // Docker creates the user `dockremap` and its subordinate ids.
        userns_remap: "default",
        no_new_privileges: true,
        live_restore: true,
        // As the agent sets on each container.
        log_driver: "json-file",
        log_opts: LogOpts {
            max_size: "10m",
            max_file: "3",
        },
        default_address_pools: [AddressPool {
            base: ADDRESS_POOL,
            size: NETWORK_PREFIX,
        }],
        // No `docker0`: every container joins a network of Brume.
        bridge: "none",
    };
    let mut json = serde_json::to_string_pretty(&config).expect("a struct serializes");
    json.push('\n');
    json
}

/// Shows whether `package` is installed, and its version: `installed 5:29.0.1-1~ubuntu…`.
pub fn package_query(package: &str) -> Cmd {
    Cmd::new(
        "dpkg-query",
        &[
            "--show",
            "--showformat=${db:Status-Status} ${Version}",
            package,
        ],
    )
}

/// The Debian version of `package`, if it is installed.
fn installed_version(host: &Host, package: &str) -> anyhow::Result<Option<String>> {
    let Some(status) = host.query_stdout(&package_query(package))? else {
        return Ok(None);
    };
    Ok(status
        .strip_prefix("installed ")
        .map(|version| version.trim().to_owned()))
}

/// Refuses a Docker that `brume install` did not install: from the distribution, from snap, or
/// with another `daemon.json`. Brume sets up a fresh server; it does not reconfigure a Docker
/// that may already run containers.
pub fn check_no_foreign_docker(host: &Host) -> anyhow::Result<()> {
    for package in FOREIGN_PACKAGES {
        if installed_version(host, package)?.is_some() {
            bail!(
                "The package {package} is installed. Brume installs Docker from \
                 download.docker.com, on a fresh server: remove {package} first."
            );
        }
    }
    if host.exists(Path::new("/snap/bin/docker")) {
        bail!(
            "Docker is installed from snap. Brume installs Docker from download.docker.com, on \
             a fresh server: remove the snap first."
        );
    }
    let present = installed_version(host, "docker-ce")?.is_some()
        || host.exists(Path::new("/usr/bin/dockerd"));
    match host.read(Path::new(DAEMON_JSON_PATH))? {
        Some(text) if text == daemon_json() => Ok(()),
        Some(_) => bail!(
            "{DAEMON_JSON_PATH} exists and is not the one Brume writes. brume install sets up a \
             fresh server: it does not rewrite another Docker configuration."
        ),
        None if present => bail!(
            "Docker is already installed, without Brume's {DAEMON_JSON_PATH}. brume install \
             sets up a fresh server: it does not reconfigure a Docker that may already run \
             containers."
        ),
        None => Ok(()),
    }
}

/// A Docker version, such as 29.0.1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

/// The epoch and the version of a Debian version of Docker, such as
/// `5:29.0.1-1~ubuntu.24.04~noble`.
fn parse_deb_version(deb: &str) -> Option<(Option<u32>, Version)> {
    let (epoch, rest) = match deb.split_once(':') {
        Some((epoch, rest)) => (Some(epoch.parse().ok()?), rest),
        None => (None, deb),
    };
    let upstream = rest.split('-').next()?;
    let mut numbers = upstream.split('.').map(|part| {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        digits.parse::<u32>().ok()
    });
    let major = numbers.next().flatten()?;
    let minor = numbers.next().flatten().unwrap_or(0);
    let patch = numbers.next().flatten().unwrap_or(0);
    Some((
        epoch,
        Version {
            major,
            minor,
            patch,
        },
    ))
}

/// The apt preference that keeps Docker on the major version of `deb`, the installed version.
/// The fixes of that version still install.
fn pin(deb: &str) -> Option<String> {
    let (epoch, version) = parse_deb_version(deb)?;
    let pattern = match epoch {
        Some(epoch) => format!("{epoch}:{}.*", version.major),
        None => format!("{}.*", version.major),
    };
    Some(format!(
        "# Written by `brume install`: Docker gets the fixes of the major version Brume was \
         installed with.\nPackage: docker-ce docker-ce-cli\nPin: version {pattern}\n\
         Pin-Priority: 990\n"
    ))
}

/// The apt source of Docker's repository, signed by `KEY`.
fn sources(platform: &Platform) -> String {
    format!(
        "# Docker's apt repository, added by `brume install`.\nTypes: deb\nURIs: \
         https://download.docker.com/linux/{distro}\nSuites: {codename}\nComponents: \
         stable\nArchitectures: {arch}\nSigned-By: {KEY_PATH}\n",
        distro = platform.distro.name(),
        codename = platform.codename,
        arch = platform.arch,
    )
}

/// Adds Docker's apt repository, with its key.
pub struct Repository {
    pub platform: Platform,
}

impl Step for Repository {
    fn name(&self) -> &str {
        "Add Docker's apt repository"
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

/// Writes `daemon.json`, before Docker's first start: its first networks already come from
/// Brume's pool, and its first containers are remapped.
pub struct Config;

impl Step for Config {
    fn name(&self) -> &str {
        "Write Docker's daemon.json"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        host.has_file(Path::new(DAEMON_JSON_PATH), &daemon_json())
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.write(Path::new(DAEMON_JSON_PATH), &daemon_json(), 0o644)?;
        if systemd::is_active(host, UNIT)? {
            host.change(&Cmd::new("systemctl", &["restart", UNIT]))?;
        }
        Ok(())
    }
}

/// Installs Docker, at least `MIN_VERSION`.
pub struct Packages;

impl Packages {
    /// The installed version of Docker, if every package is installed and Docker is recent
    /// enough.
    fn installed(host: &Host) -> anyhow::Result<Option<String>> {
        for package in &PACKAGES[1..] {
            if installed_version(host, package)?.is_none() {
                return Ok(None);
            }
        }
        let Some(deb) = installed_version(host, PACKAGES[0])? else {
            return Ok(None);
        };
        let recent = parse_deb_version(&deb).is_some_and(|(_, version)| version >= MIN_VERSION);
        Ok(recent.then_some(deb))
    }
}

impl Step for Packages {
    fn name(&self) -> &str {
        "Install Docker from download.docker.com"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(Self::installed(host)?.is_some())
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.change(&apt_get(&["update"]))?;
        let mut install = vec!["install", "--no-install-recommends"];
        install.extend_from_slice(&PACKAGES);
        host.change(&apt_get(&install))?;
        if Self::installed(host)?.is_none() {
            let found = installed_version(host, PACKAGES[0])?;
            bail!(
                "apt installed Docker {}, and Brume needs Docker 28.5.2 or later.",
                found.as_deref().unwrap_or("partly")
            );
        }
        Ok(())
    }
}

/// Pins Docker's major version: a new major version can break a client of Docker's API, so
/// Brume moves to it after testing it.
pub struct PinMajor;

impl PinMajor {
    fn expected(host: &Host) -> anyhow::Result<Option<String>> {
        Ok(Packages::installed(host)?.and_then(|deb| pin(&deb)))
    }
}

impl Step for PinMajor {
    fn name(&self) -> &str {
        "Pin Docker's major version"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        match Self::expected(host)? {
            Some(pin) => host.has_file(Path::new(PIN_PATH), &pin),
            None => Ok(false),
        }
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let Some(pin) = Self::expected(host)? else {
            bail!("Docker is not installed, or its version does not parse.");
        };
        host.write(Path::new(PIN_PATH), &pin, 0o644)
    }
}

/// Starts Docker at boot and now, and checks that it remaps the containers.
pub struct Running;

impl Running {
    fn remaps(host: &Host) -> anyhow::Result<bool> {
        let options = host.query_stdout(&security_options())?;
        Ok(options.is_some_and(|options| options.contains("name=userns")))
    }
}

/// Docker's security options, such as `["name=seccomp,profile=builtin","name=userns"]`.
pub fn security_options() -> Cmd {
    Cmd::new("docker", &["info", "--format", "{{json .SecurityOptions}}"])
}

impl Step for Running {
    fn name(&self) -> &str {
        "Start Docker, with user namespaces"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(systemd::is_enabled(host, UNIT)?
            && systemd::is_active(host, UNIT)?
            && Self::remaps(host)?)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.change(&Cmd::new("systemctl", &["enable", UNIT]))?;
        host.change(&Cmd::new("systemctl", &["restart", UNIT]))?;
        if !Self::remaps(host)? {
            bail!(
                "Docker runs without user namespaces, despite {DAEMON_JSON_PATH}. Check `docker \
                 info` and `journalctl -u docker`."
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{MIN_VERSION, Version, daemon_json, parse_deb_version, pin, sources};
    use crate::install::preflight::{Distro, Platform};

    #[test]
    fn daemon_json_is_hardened() {
        let config: serde_json::Value = serde_json::from_str(&daemon_json()).expect("JSON");
        assert_eq!(
            config,
            json!({
                "userns-remap": "default",
                "no-new-privileges": true,
                "live-restore": true,
                "log-driver": "json-file",
                "log-opts": { "max-size": "10m", "max-file": "3" },
                "default-address-pools": [{ "base": "10.213.0.0/17", "size": 24 }],
                "bridge": "none",
            })
        );
        assert!(config.get("hosts").is_none());
        assert!(daemon_json().ends_with("}\n"));
    }

    #[test]
    fn reads_docker_versions() {
        assert_eq!(
            parse_deb_version("5:29.0.1-1~ubuntu.24.04~noble"),
            Some((
                Some(5),
                Version {
                    major: 29,
                    minor: 0,
                    patch: 1
                }
            ))
        );
        let at = |deb: &str| parse_deb_version(deb).expect("parses").1;
        assert!(at("5:28.5.2-1~debian.12~bookworm") >= MIN_VERSION);
        assert!(at("5:28.5.1-1~debian.12~bookworm") < MIN_VERSION);
        assert!(at("5:30.0.0~rc.1-1~ubuntu.26.04~resolute") >= MIN_VERSION);
        assert!(at("27.5.1-0ubuntu3") < MIN_VERSION);
        assert_eq!(parse_deb_version("x:29.0.1-1"), None);
        assert_eq!(parse_deb_version(""), None);
    }

    #[test]
    fn pins_the_installed_major_version() {
        assert_eq!(
            pin("5:29.0.1-1~ubuntu.24.04~noble").expect("pins"),
            "# Written by `brume install`: Docker gets the fixes of the major version Brume was \
             installed with.\nPackage: docker-ce docker-ce-cli\nPin: version 5:29.*\n\
             Pin-Priority: 990\n"
        );
        assert!(
            pin("29.0.1-1")
                .expect("pins")
                .contains("\nPin: version 29.*\n")
        );
    }

    #[test]
    fn the_repository_matches_the_distribution() {
        let platform = Platform {
            distro: Distro::Debian,
            codename: "trixie".to_owned(),
            arch: "amd64",
        };
        assert_eq!(
            sources(&platform),
            "# Docker's apt repository, added by `brume install`.\nTypes: deb\nURIs: \
             https://download.docker.com/linux/debian\nSuites: trixie\nComponents: stable\n\
             Architectures: amd64\nSigned-By: /etc/apt/keyrings/docker.asc\n"
        );
    }
}
