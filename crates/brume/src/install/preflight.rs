//! What `brume install` checks before changing anything: root, systemd, a supported
//! distribution and architecture, SSH on the port that the firewall lets in, no Docker of
//! another origin, and no interface that the firewall would take for Docker's.

use std::collections::BTreeSet;
use std::net::IpAddr;
use std::path::Path;

use anyhow::{Context as _, bail};
use serde::Deserialize;

use super::docker;
use super::host::{Cmd, Host, NOT_FOUND};
use super::nftables::SSH_PORT;

/// Where systemd says that it runs.
const SYSTEMD_RUN_DIR: &str = "/run/systemd/system";

/// The prefix of Docker's bridges, whose traffic the firewall's input chain drops.
const DOCKER_BRIDGE_PREFIX: &str = "br-";
/// Docker's default bridge, which the firewall's input chain drops too.
const DOCKER_DEFAULT_BRIDGE: &str = "docker0";

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
    check_interfaces(host)?;
    Ok(Platform {
        distro,
        codename,
        arch,
    })
}

/// Refuses an SSH server that does not listen on port 22, outside the loopback: the firewall
/// would cut it off. Its configuration says where sshd listens, and the listening sockets catch
/// the rest, such as a drop-in of `ssh.socket` that moves the port. They also stand in for a
/// configuration that sshd cannot print, as Ubuntu's socket-activated sshd before its first
/// connection. Without an SSH server, nothing is checked: the console remains. Nor are the
/// sockets, if `ss` fails.
fn check_ssh_port(host: &Host) -> anyhow::Result<()> {
    let config = host.query(&Cmd::new("sshd", &["-T"]))?;
    if config.code == Some(NOT_FOUND) {
        return Ok(());
    }
    let ports = if config.success() {
        ssh_ports(&config.stdout)
    } else {
        BTreeSet::new()
    };
    if !ports.is_empty() && !ports.contains(SSH_PORT) {
        bail!(
            "SSH listens on port {}, and Brume's firewall lets in port {SSH_PORT} only: you would \
             lose SSH. Move SSH to port {SSH_PORT} first.",
            ports.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
    let Some(listening) = host.query_stdout(&listening_query())? else {
        return Ok(());
    };
    if !listens_outside_loopback(&listening, SSH_PORT) {
        bail!(
            "No socket listens on TCP port {SSH_PORT} outside the loopback (ss -ltn), and \
             Brume's firewall lets SSH in on port {SSH_PORT} only: you would lose SSH. Make SSH \
             listen on port {SSH_PORT} first, in sshd_config or in a drop-in of ssh.socket."
        );
    }
    Ok(())
}

/// The ports where sshd listens, from `sshd -T` (`config`): those of its `listenaddress`
/// lines, which carry their port, else its `port` lines.
fn ssh_ports(config: &str) -> BTreeSet<&str> {
    let listen: BTreeSet<&str> = config
        .lines()
        .filter_map(|line| line.strip_prefix("listenaddress "))
        // `listenaddress 10.0.0.5:2222 rdomain vrf1`.
        .filter_map(|address| address.split_whitespace().next()?.rsplit_once(':'))
        .map(|(_, port)| port)
        .collect();
    if !listen.is_empty() {
        return listen;
    }
    config
        .lines()
        .filter_map(|line| line.strip_prefix("port "))
        .map(str::trim)
        .collect()
}

/// Lists the listening TCP sockets, numeric, without a header: `LISTEN 0 4096 0.0.0.0:22
/// 0.0.0.0:*`.
fn listening_query() -> Cmd {
    Cmd::new("ss", &["-Hltn"])
}

/// Whether `ss -Hltn` (`listening`) shows a socket on `port` at an address that is not a
/// loopback one: `0.0.0.0`, `[::]`, `*` or an address of the host.
fn listens_outside_loopback(listening: &str, port: &str) -> bool {
    listening
        .lines()
        .filter_map(|line| line.split_whitespace().nth(3))
        .filter_map(|local| local.rsplit_once(':'))
        .any(|(address, local_port)| local_port == port && !is_loopback(address))
}

/// Whether `address`, as `ss` prints it (`127.0.0.53%lo`, `[::1]`, `*`), is a loopback one.
fn is_loopback(address: &str) -> bool {
    let address = address.trim_start_matches('[').trim_end_matches(']');
    let address = address.split('%').next().unwrap_or(address);
    address
        .trim_end_matches(']')
        .parse::<IpAddr>()
        .is_ok_and(|ip| ip.is_loopback())
}

/// A network interface, as `ip -j link show` lists it.
#[derive(Deserialize)]
struct Link {
    ifname: String,
}

/// Lists the network interfaces, in JSON.
fn links_query() -> Cmd {
    Cmd::new("ip", &["-j", "link", "show"])
}

/// Refuses, before Docker is installed, an interface that the firewall would take for one of
/// Docker's bridges: its input chain drops all that comes in through `br-*` and `docker0`, so
/// an uplink of that name, such as OpenStack's `br-ex`, would cut the server off. Once Docker is
/// installed, these interfaces are its own. Fails if the interfaces cannot be listed.
fn check_interfaces(host: &Host) -> anyhow::Result<()> {
    if docker::is_installed(host)? {
        return Ok(());
    }
    let links = host
        .query_stdout(&links_query())?
        .context("Cannot list the network interfaces (ip -j link show).")?;
    let links: Vec<Link> = serde_json::from_str(&links)
        .context("Cannot read the network interfaces (ip -j link show).")?;
    let Some(link) = links.iter().find(|link| {
        link.ifname.starts_with(DOCKER_BRIDGE_PREFIX) || link.ifname == DOCKER_DEFAULT_BRIDGE
    }) else {
        return Ok(());
    };
    bail!(
        "This server has a network interface named {}, and Docker is not installed yet. Brume's \
         firewall drops all that comes in through an interface named {DOCKER_BRIDGE_PREFIX}* or \
         {DOCKER_DEFAULT_BRIDGE}, Docker's bridges: through this one, you would lose SSH and the \
         server's traffic. Brume installs on a server whose interfaces have other names.",
        link.ifname
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
            Output::ok(
                "port 22\naddressfamily any\nlistenaddress [::]:22\nlistenaddress 0.0.0.0:22\n",
            ),
        );
        fake.answer(&super::listening_query(), Output::ok(LISTENING));
        fake.answer(&super::links_query(), Output::ok(LINKS));
        fake
    }

    /// `ip -j link show` on a fresh server: the loopback and the uplink.
    const LINKS: &str = r#"[{"ifindex":1,"ifname":"lo","flags":["LOOPBACK","UP"]},{"ifindex":2,"ifname":"eth0","flags":["UP"]}]"#;

    /// `ss -Hltn` with SSH on port 22, and a resolver on the loopback.
    const LISTENING: &str = "LISTEN 0      4096   127.0.0.53%lo:53         0.0.0.0:*\nLISTEN \
                             0      4096         0.0.0.0:22         0.0.0.0:*\nLISTEN 0      \
                             4096            [::]:22            [::]:*\n";

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
    fn refuses_ssh_away_from_port_22() {
        // `ListenAddress` with a port: sshd ignores `Port` there.
        let listen_address = fresh();
        listen_address.answer(
            &Cmd::new("sshd", &["-T"]),
            Output::ok("port 22\nlistenaddress 10.0.0.5:2222\n"),
        );
        // ssh.socket moved by a drop-in: sshd's configuration still says 22.
        let socket = fresh();
        socket.answer(
            &super::listening_query(),
            Output::ok("LISTEN 0 4096 0.0.0.0:2222 0.0.0.0:*\nLISTEN 0 4096 [::]:2222 [::]:*\n"),
        );
        // Ubuntu's socket-activated sshd cannot print its configuration before a first
        // connection: the sockets still tell.
        let unreadable = fresh();
        unreadable.answer(&Cmd::new("sshd", &["-T"]), Output::failed(255));
        unreadable.answer(
            &super::listening_query(),
            Output::ok("LISTEN 0 4096 [::]:2222 [::]:*\n"),
        );
        // Port 22 on the loopback only.
        let loopback = fresh();
        loopback.answer(
            &super::listening_query(),
            Output::ok("LISTEN 0 128 127.0.0.1:22 0.0.0.0:*\nLISTEN 0 128 [::1]:22 [::]:*\n"),
        );
        for (needle, fake) in [
            ("SSH listens on port 2222", listen_address),
            ("No socket listens on TCP port 22", socket),
            ("No socket listens on TCP port 22", unreadable),
            ("No socket listens on TCP port 22", loopback),
        ] {
            let error = check(&Host::new(Box::new(fake))).expect_err(needle);
            assert!(error.to_string().contains(needle), "{needle}: {error}");
        }
    }

    #[test]
    fn refuses_an_interface_that_the_firewall_takes_for_docker() {
        for name in ["br-ex", "docker0"] {
            let fake = fresh();
            fake.answer(
                &super::links_query(),
                Output::ok(&format!(
                    r#"[{{"ifindex":1,"ifname":"lo"}},{{"ifindex":2,"ifname":"{name}"}}]"#
                )),
            );
            let error = check(&Host::new(Box::new(fake))).expect_err(name);
            assert!(
                error
                    .to_string()
                    .starts_with(&format!("This server has a network interface named {name}")),
                "{error}"
            );
        }
        // A bridge of another name passes. A server whose links cannot be listed is refused:
        // the check fails closed.
        let br0 = fresh();
        br0.answer(
            &super::links_query(),
            Output::ok(r#"[{"ifname":"lo"},{"ifname":"br0"},{"ifname":"virbr0"}]"#),
        );
        assert!(check(&Host::new(Box::new(br0))).is_ok());
        let unreadable = fresh();
        unreadable.answer(&super::links_query(), Output::failed(1));
        assert!(check(&Host::new(Box::new(unreadable))).is_err());

        // Once Docker is installed, its bridges are Brume's: a rerun passes.
        let installed = fresh();
        installed.put_file("/etc/docker/daemon.json", &docker::daemon_json());
        installed.answer(
            &docker::package_query("docker-ce"),
            Output::ok("installed 5:29.0.1-1~ubuntu.24.04~noble"),
        );
        installed.answer(
            &super::links_query(),
            Output::ok(r#"[{"ifname":"lo"},{"ifname":"eth0"},{"ifname":"br-0a1b2c3d4e5f"}]"#),
        );
        assert!(check(&Host::new(Box::new(installed))).is_ok());
    }

    #[test]
    fn ssh_on_22_and_another_port_passes() {
        let fake = fresh();
        fake.answer(
            &Cmd::new("sshd", &["-T"]),
            Output::ok("port 22\nport 2222\n"),
        );
        assert!(check(&Host::new(Box::new(fake.clone()))).is_ok());
        // A configuration that cannot be read, with SSH on port 22.
        fake.answer(&Cmd::new("sshd", &["-T"]), Output::failed(255));
        assert!(check(&Host::new(Box::new(fake.clone()))).is_ok());
        // No SSH server at all: the console of the host remains.
        fake.answer(&Cmd::new("sshd", &["-T"]), Output::failed(127));
        fake.answer(&super::listening_query(), Output::ok(""));
        assert!(check(&Host::new(Box::new(fake))).is_ok());
    }
}
