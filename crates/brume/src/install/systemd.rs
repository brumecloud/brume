//! The systemd side of a host: the directories (`tmpfiles.d`), one hardened unit per service,
//! and their start.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use brume_runtime::peer::{AGENT_USER, API_USER, ENGINE_USER, TELEMETRY_USER};

use super::host::{Cmd, DirMeta, Host};
use super::users::SHARED_GROUP;
use super::{Step, nftables};

/// Where `brume install` writes its units.
const UNIT_DIR: &str = "/etc/systemd/system";

/// The directories of a host, created at each boot by `systemd-tmpfiles`, which also resets
/// their owners and modes.
const TMPFILES_PATH: &str = "/etc/tmpfiles.d/brume.conf";

/// The binary that `install.sh` links.
const BRUME: &str = "/usr/local/bin/brume";

/// A directory of a host: its path, owner and mode.
struct Dir {
    path: &'static str,
    user: &'static str,
    group: &'static str,
    mode: u32,
}

/// The directories, parents first.
///
/// `/run/brume` holds one socket directory per listening service, owned by it and not
/// writable by the others: no service can remove another's socket or listen in its place. Each
/// is setgid, so that each socket gets the group that the four users share and the `0660`
/// sockets open to the other services.
const DIRS: [Dir; 9] = [
    Dir {
        path: "/run/brume",
        user: "root",
        group: "root",
        mode: 0o755,
    },
    Dir {
        path: "/run/brume/engine",
        user: ENGINE_USER,
        group: SHARED_GROUP,
        mode: 0o2750,
    },
    Dir {
        path: "/run/brume/telemetry",
        user: TELEMETRY_USER,
        group: SHARED_GROUP,
        mode: 0o2750,
    },
    Dir {
        path: "/var/lib/brume",
        user: "root",
        group: "root",
        mode: 0o755,
    },
    Dir {
        path: "/var/lib/brume/engine",
        user: ENGINE_USER,
        group: ENGINE_USER,
        mode: 0o700,
    },
    Dir {
        path: "/var/lib/brume/bucket",
        user: ENGINE_USER,
        group: ENGINE_USER,
        mode: 0o700,
    },
    Dir {
        path: "/var/lib/brume/telemetry",
        user: TELEMETRY_USER,
        group: TELEMETRY_USER,
        mode: 0o700,
    },
    // Traefik's configuration, which the agent writes, without secrets, and Traefik reads as
    // its remapped user.
    Dir {
        path: "/var/lib/brume/agent",
        user: AGENT_USER,
        group: AGENT_USER,
        mode: 0o755,
    },
    Dir {
        path: "/var/lib/brume/agent/traefik",
        user: AGENT_USER,
        group: AGENT_USER,
        mode: 0o755,
    },
];

/// `/etc/tmpfiles.d/brume.conf`.
fn tmpfiles() -> String {
    let mut text = String::from(
        "# Brume's directories, written by `brume install`: the next run overwrites any \
         change.\n",
    );
    for dir in &DIRS {
        writeln!(
            text,
            "d {} {:04o} {} {} -",
            dir.path, dir.mode, dir.user, dir.group
        )
        .expect("writes to a String");
    }
    text
}

/// Creates the directories, with their owners and modes.
pub struct Directories;

impl Step for Directories {
    fn name(&self) -> &str {
        "Create the directories in /var/lib/brume and /run/brume"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        if !host.has_file(Path::new(TMPFILES_PATH), &tmpfiles())? {
            return Ok(false);
        }
        for dir in &DIRS {
            if host.dir(Path::new(dir.path))? != Some(DirMeta::new(dir.user, dir.group, dir.mode)) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.write(Path::new(TMPFILES_PATH), &tmpfiles(), 0o644)?;
        host.change(&Cmd::new("systemd-tmpfiles", &["--create", TMPFILES_PATH]))?;
        Ok(())
    }
}

/// A service of Brume, run by systemd.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    Telemetry,
    Engine,
    Agent,
    Api,
}

impl Service {
    /// In start order.
    pub const ALL: [Self; 4] = [Self::Telemetry, Self::Engine, Self::Agent, Self::Api];

    /// The subcommand of `brume`.
    fn command(self) -> &'static str {
        match self {
            Self::Telemetry => "telemetry",
            Self::Engine => "engine",
            Self::Agent => "agent",
            Self::Api => "api",
        }
    }

    pub fn unit(self) -> String {
        format!("brume-{}.service", self.command())
    }

    pub fn user(self) -> &'static str {
        match self {
            Self::Telemetry => TELEMETRY_USER,
            Self::Engine => ENGINE_USER,
            Self::Agent => AGENT_USER,
            Self::Api => API_USER,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Telemetry => "Brume telemetry: stores and serves the logs of the containers",
            Self::Engine => "Brume engine: the event log and the state",
            Self::Agent => "Brume agent: drives Docker",
            Self::Api => "Brume API: the panel, on 127.0.0.1",
        }
    }

    /// The `[Unit]` dependencies. The agent never runs without the firewall, nor before Docker.
    fn dependencies(self) -> &'static str {
        match self {
            Self::Telemetry => "",
            Self::Engine => "Wants=brume-telemetry.service\nAfter=brume-telemetry.service\n",
            Self::Agent => {
                "Requires=brume-nftables.service docker.service\nAfter=brume-nftables.service \
                 docker.service\nWants=brume-engine.service brume-telemetry.service\n\
                 After=brume-engine.service brume-telemetry.service\n"
            }
            Self::Api => "Wants=brume-engine.service\nAfter=brume-engine.service\n",
        }
    }

    /// What the service may write, besides creating its sockets.
    fn read_write_paths(self) -> &'static str {
        match self {
            Self::Telemetry => "/var/lib/brume/telemetry /run/brume/telemetry",
            Self::Engine => "/var/lib/brume/engine /var/lib/brume/bucket /run/brume/engine",
            Self::Agent => "/var/lib/brume/agent",
            Self::Api => "",
        }
    }

    /// The network the service may use. Unix sockets live in the file system: `PrivateNetwork`
    /// does not cut them. The agent speaks to Docker, the engine and telemetry over unix sockets
    /// only, and Docker pulls the images: the agent's HTTP probes run in a container.
    fn network(self) -> &'static str {
        match self {
            Self::Telemetry => "PrivateNetwork=yes\nRestrictAddressFamilies=AF_UNIX\n",
            Self::Agent => "RestrictAddressFamilies=AF_UNIX\nIPAddressDeny=any\n",
            Self::Engine => "RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6\n",
            Self::Api => {
                "RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6\nIPAddressDeny=any\n\
                 IPAddressAllow=localhost\n"
            }
        }
    }

    /// The limits of the service: telemetry has a ceiling, so that a flood of logs cannot
    /// starve the server; the others are spared by the OOM killer.
    fn resources(self) -> &'static str {
        match self {
            Self::Telemetry => "MemoryMax=512M\nCPUQuota=50%\n",
            Self::Engine | Self::Agent | Self::Api => "OOMScoreAdjust=-500\n",
        }
    }

    /// The unit file. Fixed values only: nothing from the user or the host.
    pub fn unit_text(self) -> String {
        let read_write = match self.read_write_paths() {
            "" => String::new(),
            paths => format!("ReadWritePaths={paths}\n"),
        };
        format!(
            "# Written by `brume install`: the next run overwrites any change.\n\
             [Unit]\n\
             Description={description}\n\
             Documentation=https://brume.dev/docs\n\
             {dependencies}\
             \n\
             [Service]\n\
             Type=exec\n\
             ExecStart={BRUME} {command} --system\n\
             User={user}\n\
             Group={user}\n\
             Restart=on-failure\n\
             RestartSec=2s\n\
             UMask=0077\n\
             {resources}\
             \n\
             NoNewPrivileges=yes\n\
             CapabilityBoundingSet=\n\
             AmbientCapabilities=\n\
             ProtectSystem=strict\n\
             ProtectHome=yes\n\
             PrivateTmp=yes\n\
             PrivateDevices=yes\n\
             {read_write}\
             ProtectKernelTunables=yes\n\
             ProtectKernelModules=yes\n\
             ProtectKernelLogs=yes\n\
             ProtectControlGroups=yes\n\
             ProtectClock=yes\n\
             ProtectHostname=yes\n\
             ProtectProc=invisible\n\
             RestrictNamespaces=yes\n\
             RestrictRealtime=yes\n\
             RestrictSUIDSGID=yes\n\
             LockPersonality=yes\n\
             MemoryDenyWriteExecute=yes\n\
             RemoveIPC=yes\n\
             SystemCallArchitectures=native\n\
             SystemCallFilter=@system-service\n\
             SystemCallErrorNumber=EPERM\n\
             {network}\
             \n\
             [Install]\n\
             WantedBy=multi-user.target\n",
            description = self.description(),
            dependencies = self.dependencies(),
            command = self.command(),
            user = self.user(),
            resources = self.resources(),
            network = self.network(),
        )
    }
}

/// Writes the units of the services, and restarts those that run with an older unit.
pub struct Units;

impl Step for Units {
    fn name(&self) -> &str {
        "Write the systemd units of the services"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        for service in Service::ALL {
            if !host.has_file(&unit_path(&service.unit()), &service.unit_text())? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let mut changed = Vec::new();
        for service in Service::ALL {
            let path = unit_path(&service.unit());
            let text = service.unit_text();
            if !host.has_file(&path, &text)? {
                host.write(&path, &text, 0o644)?;
                changed.push(service.unit());
            }
        }
        host.change(&daemon_reload())?;
        if !changed.is_empty() {
            // Restarts those that run, only.
            let mut restart = vec!["try-restart"];
            restart.extend(changed.iter().map(String::as_str));
            host.change(&Cmd::new("systemctl", &restart))?;
        }
        Ok(())
    }
}

/// Starts the services, now and at each boot.
pub struct Services;

impl Step for Services {
    fn name(&self) -> &str {
        "Start the services"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        for service in Service::ALL {
            let unit = service.unit();
            if !is_enabled(host, &unit)? || !is_active(host, &unit)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let units: Vec<String> = Service::ALL.iter().map(|service| service.unit()).collect();
        let mut enable = vec!["enable", "--now"];
        enable.extend(units.iter().map(String::as_str));
        host.change(&Cmd::new("systemctl", &enable))?;
        Ok(())
    }
}

/// The path of the unit `unit` written by `brume install`.
pub fn unit_path(unit: &str) -> PathBuf {
    Path::new(UNIT_DIR).join(unit)
}

pub fn daemon_reload() -> Cmd {
    Cmd::new("systemctl", &["daemon-reload"])
}

/// Prints `enabled` if `unit` starts at boot.
pub fn is_enabled_query(unit: &str) -> Cmd {
    Cmd::new("systemctl", &["is-enabled", unit])
}

/// Prints `active` if `unit` runs.
pub fn is_active_query(unit: &str) -> Cmd {
    Cmd::new("systemctl", &["is-active", unit])
}

pub fn is_enabled(host: &Host, unit: &str) -> anyhow::Result<bool> {
    Ok(host.query_stdout(&is_enabled_query(unit))?.as_deref() == Some("enabled"))
}

pub fn is_active(host: &Host, unit: &str) -> anyhow::Result<bool> {
    Ok(host.query_stdout(&is_active_query(unit))?.as_deref() == Some("active"))
}

/// The units whose start the install checks at the end: the firewall, Docker and the services.
pub fn checked_units() -> Vec<String> {
    let mut units = vec![nftables::UNIT.to_owned(), "docker.service".to_owned()];
    units.extend(Service::ALL.iter().map(|service| service.unit()));
    units
}

#[cfg(test)]
mod tests {
    use super::{DIRS, Service, tmpfiles};

    /// The values of `key` in the unit `text`.
    fn values<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
        text.lines()
            .filter_map(|line| line.strip_prefix(key)?.strip_prefix('='))
            .collect()
    }

    #[test]
    fn units_are_hardened() {
        for service in Service::ALL {
            let text = service.unit_text();
            let one = |key: &str| {
                let found = values(&text, key);
                assert_eq!(found.len(), 1, "{service:?}: {key} = {found:?}");
                found[0]
            };
            assert_eq!(one("User"), service.user(), "{service:?}");
            assert!(one("User").starts_with("brume-"));
            assert_eq!(one("NoNewPrivileges"), "yes");
            assert_eq!(one("ProtectSystem"), "strict");
            assert_eq!(one("ProtectHome"), "yes");
            assert_eq!(one("PrivateTmp"), "yes");
            assert_eq!(one("CapabilityBoundingSet"), "");
            assert_eq!(one("AmbientCapabilities"), "");
            assert_eq!(
                one("ExecStart"),
                format!("/usr/local/bin/brume {} --system", service.command())
            );
            // Each service writes in its own directory only.
            for path in values(&text, "ReadWritePaths")
                .iter()
                .flat_map(|paths| paths.split(' '))
            {
                assert!(
                    path.starts_with("/run/brume/") || path.starts_with("/var/lib/brume/"),
                    "{service:?}: {path}"
                );
            }
        }
        let agent = Service::Agent.unit_text();
        assert_eq!(
            values(&agent, "Requires"),
            ["brume-nftables.service docker.service"]
        );
        assert!(values(&agent, "After").contains(&"brume-nftables.service docker.service"));
        let telemetry = Service::Telemetry.unit_text();
        assert_eq!(values(&telemetry, "PrivateNetwork"), ["yes"]);
        assert_eq!(values(&telemetry, "MemoryMax"), ["512M"]);
        // The agent speaks over unix sockets only: Docker's, the engine's, telemetry's. Its
        // probes run in containers.
        assert_eq!(values(&agent, "RestrictAddressFamilies"), ["AF_UNIX"]);
        assert_eq!(values(&agent, "IPAddressDeny"), ["any"]);
        assert!(values(&agent, "IPAddressAllow").is_empty());
        let api = Service::Api.unit_text();
        assert_eq!(values(&api, "IPAddressDeny"), ["any"]);
        assert_eq!(values(&api, "IPAddressAllow"), ["localhost"]);
        assert!(values(&api, "ReadWritePaths").is_empty());
        for service in [Service::Telemetry, Service::Engine, Service::Api] {
            assert!(values(&service.unit_text(), "Requires").is_empty());
        }
    }

    #[test]
    fn each_listener_owns_its_sockets_directory() {
        let conf = tmpfiles();
        assert!(conf.contains("\nd /run/brume 0755 root root -\n"), "{conf}");
        assert!(
            conf.contains("\nd /run/brume/engine 2750 brume-engine brume -\n"),
            "{conf}"
        );
        assert!(
            conf.contains("\nd /run/brume/telemetry 2750 brume-telemetry brume -\n"),
            "{conf}"
        );
        assert!(conf.contains("\nd /var/lib/brume/engine 0700 brume-engine brume-engine -\n"));
        assert!(conf.contains("\nd /var/lib/brume/agent/traefik 0755 brume-agent brume-agent -\n"));
        assert_eq!(conf.lines().count(), DIRS.len() + 1);
    }
}
