//! Docker, from Docker's own apt repository, configured before its first start: user
//! namespaces, no new privileges, bounded logs, Brume's address pools, no default bridge and no
//! TCP socket.

use std::fmt;
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

/// The oldest Docker Brume runs on, 28.5.2: it came out with runc 1.3.3. runc itself comes with
/// `containerd.io`, and is checked apart.
const MIN_VERSION: Version = Version {
    major: 28,
    minor: 5,
    patch: 2,
};

/// The oldest runc Brume runs on: 1.3.3 fixes the container escapes of November 2025
/// (CVE-2025-31133, CVE-2025-52565, CVE-2025-52881).
const MIN_RUNC: Version = Version {
    major: 1,
    minor: 3,
    patch: 3,
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
    let present = is_installed(host)?;
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

/// Whether Docker is installed, by any means.
pub fn is_installed(host: &Host) -> anyhow::Result<bool> {
    Ok(installed_version(host, "docker-ce")?.is_some()
        || host.exists(Path::new("/usr/bin/dockerd")))
}

/// A version of Docker or runc, such as 29.0.1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// The version that `text` starts with, such as `29.0.1` or `1.4.0-rc.1`: the digits that
/// start each of its first three dot-separated parts.
fn parse_version(text: &str) -> Option<Version> {
    let mut numbers = text.split('.').map(|part| {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        digits.parse::<u32>().ok()
    });
    let major = numbers.next().flatten()?;
    let minor = numbers.next().flatten().unwrap_or(0);
    let patch = numbers.next().flatten().unwrap_or(0);
    Some(Version {
        major,
        minor,
        patch,
    })
}

/// The epoch and the version of a Debian version of Docker, such as
/// `5:29.0.1-1~ubuntu.24.04~noble`.
fn parse_deb_version(deb: &str) -> Option<(Option<u32>, Version)> {
    let (epoch, rest) = match deb.split_once(':') {
        Some((epoch, rest)) => (Some(epoch.parse().ok()?), rest),
        None => (None, deb),
    };
    Some((epoch, parse_version(rest.split('-').next()?)?))
}

/// Prints runc's version, `runc version 1.3.3`, then its commit and its dependencies.
pub fn runc_version_query() -> Cmd {
    Cmd::new("runc", &["--version"])
}

/// The version of runc, from the first line of `runc --version` (`output`).
fn parse_runc_version(output: &str) -> Option<Version> {
    parse_version(output.lines().next()?.strip_prefix("runc version ")?.trim())
}

/// The version of the runc that Docker runs containers with, if it tells.
fn runc_version(host: &Host) -> anyhow::Result<Option<Version>> {
    Ok(host
        .query_stdout(&runc_version_query())?
        .and_then(|output| parse_runc_version(&output)))
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

/// Installs Docker, at least `MIN_VERSION`, with runc at least `MIN_RUNC`.
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
        Ok(Self::installed(host)?.is_some()
            && runc_version(host)?.is_some_and(|version| version >= MIN_RUNC))
    }

    /// Installs the packages, or upgrades them: an older runc comes with an older
    /// `containerd.io`.
    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.change(&apt_get(&["update"]))?;
        let mut install = vec!["install", "--no-install-recommends"];
        install.extend_from_slice(&PACKAGES);
        host.change(&apt_get(&install))?;
        if Self::installed(host)?.is_none() {
            let found = installed_version(host, PACKAGES[0])?;
            bail!(
                "apt installed Docker {}, and Brume needs Docker {MIN_VERSION} or later.",
                found.as_deref().unwrap_or("partly")
            );
        }
        match runc_version(host)? {
            Some(version) if version >= MIN_RUNC => Ok(()),
            Some(version) => bail!(
                "Docker runs containers with runc {version}, and Brume needs runc {MIN_RUNC} or \
                 later: older versions let a container escape to the host (CVE-2025-31133, \
                 CVE-2025-52565, CVE-2025-52881). runc comes with the package containerd.io."
            ),
            None => bail!(
                "Docker is installed, but brume install cannot tell the version of runc (runc \
                 --version), and Brume needs runc {MIN_RUNC} or later."
            ),
        }
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

    use super::{
        MIN_RUNC, MIN_VERSION, PACKAGES, Packages, Version, daemon_json, package_query,
        parse_deb_version, parse_runc_version, pin, runc_version_query, sources,
    };
    use crate::install::Step as _;
    use crate::install::host::fake::FakeSystem;
    use crate::install::host::{Host, Output};
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

    #[test]
    fn reads_runc_versions() {
        let output = "runc version 1.3.3\ncommit: v1.3.3-0-gd842d771\nspec: 1.2.1\ngo: \
                      go1.24.9\nlibseccomp: 2.5.5\n";
        assert_eq!(
            parse_runc_version(output),
            Some(Version {
                major: 1,
                minor: 3,
                patch: 3
            })
        );
        let at = |output: &str| parse_runc_version(output).expect("parses");
        assert!(at(output) >= MIN_RUNC);
        assert!(at("runc version 1.3.2\n") < MIN_RUNC);
        assert!(at("runc version 1.2.8\n") < MIN_RUNC);
        assert!(at("runc version 1.4.0-rc.1\ncommit: x\n") >= MIN_RUNC);
        assert!(at("runc version 1.5.2\n") >= MIN_RUNC);
        assert_eq!(parse_runc_version("commit: v1.3.3\n"), None);
        assert_eq!(parse_runc_version(""), None);
    }

    /// A host with Docker 29 installed, and `runc` at `runc`, if it answers.
    fn with_runc(runc: Option<&str>) -> FakeSystem {
        let fake = FakeSystem::default();
        for package in PACKAGES {
            fake.answer(
                &package_query(package),
                Output::ok("installed 5:29.0.1-1~ubuntu.24.04~noble"),
            );
        }
        if let Some(version) = runc {
            fake.answer(
                &runc_version_query(),
                Output::ok(&format!(
                    "runc version {version}\ncommit: v{version}-0-g0\n"
                )),
            );
        }
        fake
    }

    #[test]
    fn an_old_runc_stops_the_install() {
        let host = Host::new(Box::new(with_runc(Some("1.3.3"))));
        assert!(Packages.is_done(&host).expect("reads"));

        for (runc, needle) in [
            (Some("1.3.2"), "runc 1.3.2"),
            (None, "cannot tell the version of runc"),
        ] {
            let fake = with_runc(runc);
            let mut host = Host::new(Box::new(fake.clone()));
            assert!(!Packages.is_done(&host).expect("reads"), "{needle}");
            // apt runs, in case it brings a newer containerd.io, then the install stops.
            let error = Packages.apply(&mut host).expect_err(needle);
            let error = error.to_string();
            assert!(error.contains(needle), "{error}");
            assert!(error.contains("runc 1.3.3 or later"), "{error}");
            assert!(
                fake.changes()
                    .iter()
                    .any(|change| change.contains("install --no-install-recommends")),
            );
        }
    }
}
