//! What `brume install` checks before changing anything: root, systemd, a supported
//! distribution and architecture, no Docker of another origin, and SSH on the port that the
//! firewall lets in.

use std::path::Path;

use anyhow::{Context as _, bail};

use super::docker;
use super::host::{Cmd, Host};
use super::nftables::SSH_PORT;

/// Where systemd says that it runs.
const SYSTEMD_RUN_DIR: &str = "/run/systemd/system";

/// The distributions Brume runs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distro {
    Ubuntu,
    Debian,
}

impl Distro {
    /// Its name in Docker's repository URLs.
    pub fn name(self) -> &'static str {
        match self {
            Self::Ubuntu => "ubuntu",
            Self::Debian => "debian",
        }
    }
}

/// The host, as the steps need to know it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Platform {
    pub distro: Distro,
    /// The release's code name, such as `noble`: the suite of apt repositories.
    pub codename: String,
    /// `amd64` or `arm64`, as dpkg names it.
    pub arch: &'static str,
}

/// The fields of `/etc/os-release` that matter here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct OsRelease {
    id: String,
    version_id: String,
    codename: String,
    pretty_name: String,
}

/// Reads `/etc/os-release`: `KEY=value` lines, the value maybe quoted.
fn parse_os_release(text: &str) -> OsRelease {
    let mut release = OsRelease::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .or_else(|| {
                value
                    .strip_prefix('\'')
                    .and_then(|value| value.strip_suffix('\''))
            })
            .unwrap_or(value)
            .to_owned();
        match key.trim() {
            "ID" => release.id = value,
            "VERSION_ID" => release.version_id = value,
            "VERSION_CODENAME" => release.codename = value,
            "PRETTY_NAME" => release.pretty_name = value,
            _ => {}
        }
    }
    release
}

/// The distribution of `os_release` (the contents of `/etc/os-release`), and its code name, if
/// Brume supports it: Ubuntu 24.04 or 26.04, Debian 12 or 13.
fn supported(os_release: &str) -> anyhow::Result<(Distro, String)> {
    let release = parse_os_release(os_release);
    let distro = match (release.id.as_str(), release.version_id.as_str()) {
        ("ubuntu", "24.04" | "26.04") => Distro::Ubuntu,
        ("debian", "12" | "13") => Distro::Debian,
        _ => {
            let name = if release.pretty_name.is_empty() {
                format!("{} {}", release.id, release.version_id)
            } else {
                release.pretty_name
            };
            bail!(
                "Brume runs on Ubuntu 24.04 or 26.04, or Debian 12 or 13. This server runs {}.",
                name.trim()
            );
        }
    };
    // It becomes a line of an apt source.
    if release.codename.is_empty() || !release.codename.bytes().all(|b| b.is_ascii_lowercase()) {
        bail!(
            "/etc/os-release has no valid VERSION_CODENAME ({:?}).",
            release.codename
        );
    }
    Ok((distro, release.codename))
}

/// Checks that `brume install` can run here, before it changes anything, and returns the
/// platform.
pub fn check(host: &Host) -> anyhow::Result<Platform> {
    if host.query_stdout(&Cmd::new("id", &["-u"]))?.as_deref() != Some("0") {
        bail!("Run brume install as root: sudo brume install …");
    }
    if !host.exists(Path::new(SYSTEMD_RUN_DIR)) {
        bail!("brume install needs systemd, and systemd is not running here.");
    }
    let os_release = host
        .read(Path::new("/etc/os-release"))?
        .context("/etc/os-release is missing: cannot tell the distribution")?;
    let (distro, codename) = supported(&os_release)?;
    let arch = match host
        .query_stdout(&Cmd::new("dpkg", &["--print-architecture"]))?
        .as_deref()
    {
        Some("amd64") => "amd64",
        Some("arm64") => "arm64",
        other => bail!(
            "Brume runs on amd64 or arm64. This server is {}.",
            other.unwrap_or("of an unknown architecture")
        ),
    };
    check_ssh_port(host)?;
    docker::check_no_foreign_docker(host)?;
    Ok(Platform {
        distro,
        codename,
        arch,
    })
}

/// Refuses an SSH server that does not listen on port 22: the firewall would cut it off. An
/// SSH server whose configuration cannot be read, or none, is not checked.
fn check_ssh_port(host: &Host) -> anyhow::Result<()> {
    let Some(config) = host.query_stdout(&Cmd::new("sshd", &["-T"]))? else {
        return Ok(());
    };
    let ports: Vec<&str> = config
        .lines()
        .filter_map(|line| line.strip_prefix("port "))
        .map(str::trim)
        .collect();
    if ports.is_empty() || ports.contains(&SSH_PORT) {
        return Ok(());
    }
    bail!(
        "SSH listens on port {}, and Brume's firewall lets in port {SSH_PORT} only: you would \
         lose SSH. Move SSH to port {SSH_PORT} first.",
        ports.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::{Distro, Platform, check, supported};
    use crate::install::docker;
    use crate::install::host::fake::FakeSystem;
    use crate::install::host::{Cmd, Host, Output};

    const UBUNTU_2404: &str = r#"PRETTY_NAME="Ubuntu 24.04.5 LTS"
NAME="Ubuntu"
VERSION_ID="24.04"
VERSION="24.04.5 LTS (Noble Numbat)"
VERSION_CODENAME=noble
ID=ubuntu
ID_LIKE=debian
UBUNTU_CODENAME=noble
"#;

    #[test]
    fn os_release_supported() {
        assert_eq!(
            supported(UBUNTU_2404).expect("supported"),
            (Distro::Ubuntu, "noble".to_owned())
        );
        let ubuntu_2604 = UBUNTU_2404
            .replace("24.04", "26.04")
            .replace("noble", "resolute");
        assert_eq!(
            supported(&ubuntu_2604).expect("supported"),
            (Distro::Ubuntu, "resolute".to_owned())
        );
        for (version, codename) in [("12", "bookworm"), ("13", "trixie")] {
            let debian = format!(
                "PRETTY_NAME=\"Debian GNU/Linux {version} ({codename})\"\nNAME=\"Debian \
                 GNU/Linux\"\nVERSION_ID=\"{version}\"\nVERSION_CODENAME={codename}\nID=debian\n"
            );
            assert_eq!(
                supported(&debian).expect("supported"),
                (Distro::Debian, codename.to_owned())
            );
        }
        let ubuntu_2204 = UBUNTU_2404
            .replace("24.04.5", "22.04.4")
            .replace("24.04", "22.04")
            .replace("noble", "jammy");
        let error = supported(&ubuntu_2204).expect_err("too old");
        assert_eq!(
            error.to_string(),
            "Brume runs on Ubuntu 24.04 or 26.04, or Debian 12 or 13. This server runs Ubuntu \
             22.04.4 LTS."
        );
        let fedora = "NAME=\"Fedora Linux\"\nVERSION_ID=42\nID=fedora\n";
        let error = supported(fedora).expect_err("not Debian or Ubuntu");
        assert!(error.to_string().contains("fedora 42"), "{error}");
        let injected = UBUNTU_2404.replace("=noble\n", "=\"noble main\nX\"\n");
        assert!(supported(&injected).is_err());
    }

    /// A fresh Ubuntu 24.04 server, on arm64, with SSH on port 22.
    fn fresh() -> FakeSystem {
        let fake = FakeSystem::default();
        fake.answer(&Cmd::new("id", &["-u"]), Output::ok("0\n"));
        fake.put_path("/run/systemd/system");
        fake.put_file("/etc/os-release", UBUNTU_2404);
        fake.answer(
            &Cmd::new("dpkg", &["--print-architecture"]),
            Output::ok("arm64\n"),
        );
        fake.answer(
            &Cmd::new("sshd", &["-T"]),
            Output::ok("port 22\naddressfamily any\n"),
        );
        fake
    }

    #[test]
    fn a_fresh_server_passes() {
        let host = Host::new(Box::new(fresh()));
        assert_eq!(
            check(&host).expect("passes"),
            Platform {
                distro: Distro::Ubuntu,
                codename: "noble".to_owned(),
                arch: "arm64",
            }
        );
    }

    #[test]
    fn refuses_what_would_break() {
        let not_root = fresh();
        not_root.answer(&Cmd::new("id", &["-u"]), Output::ok("1000\n"));
        let no_systemd = FakeSystem::default();
        no_systemd.answer(&Cmd::new("id", &["-u"]), Output::ok("0\n"));
        let riscv = fresh();
        riscv.answer(
            &Cmd::new("dpkg", &["--print-architecture"]),
            Output::ok("riscv64\n"),
        );
        let ssh_elsewhere = fresh();
        ssh_elsewhere.answer(&Cmd::new("sshd", &["-T"]), Output::ok("port 2222\n"));
        let other_daemon_json = fresh();
        other_daemon_json.put_file("/etc/docker/daemon.json", "{}\n");
        let docker_io = fresh();
        docker_io.answer(
            &docker::package_query("docker.io"),
            Output::ok("installed 26.1.3-0ubuntu1"),
        );
        for (needle, fake) in [
            ("as root", not_root),
            ("systemd", no_systemd),
            ("amd64 or arm64", riscv),
            ("SSH listens on port 2222", ssh_elsewhere),
            ("daemon.json", other_daemon_json),
            ("docker.io", docker_io),
        ] {
            let host = Host::new(Box::new(fake));
            let error = check(&host).expect_err(needle);
            assert!(error.to_string().contains(needle), "{needle}: {error}");
        }
    }

    #[test]
    fn ssh_on_22_and_another_port_passes() {
        let fake = fresh();
        fake.answer(
            &Cmd::new("sshd", &["-T"]),
            Output::ok("port 22\nport 2222\n"),
        );
        assert!(check(&Host::new(Box::new(fake.clone()))).is_ok());
        // No SSH server at all: the console of the host remains.
        fake.answer(&Cmd::new("sshd", &["-T"]), Output::failed(127));
        assert!(check(&Host::new(Box::new(fake))).is_ok());
    }
}
