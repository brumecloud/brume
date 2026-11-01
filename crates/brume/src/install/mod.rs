//! `brume install`, run as root on a fresh server: turns it into a Brume host. Brume's
//! firewall, Docker with user namespaces, one system user per service, hardened systemd units,
//! then a login link to the panel.
//!
//! Each step checks whether it is done before doing anything: running the install again, after
//! a failure or on an installed host, does only what is missing, then prints a new login link.
//! The engine's master key is never touched: the engine creates it at its first start.

mod api_config;
mod docker;
mod host;
mod nftables;
mod preflight;
mod steps;
mod systemd;
mod users;

use std::io::{BufRead as _, Read as _, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use brume_api::{ApiFileConfig, SYSTEM_CONFIG_PATH};
use brume_runtime::paths::Paths;
use clap::{Args, ValueEnum};

use self::host::Host;
use crate::login;

/// The longest wait for the engine's first login link, after the services start.
const LINK_TIMEOUT: Duration = Duration::from_secs(60);

/// How often the engine is asked while it starts.
const LINK_POLL: Duration = Duration::from_millis(500);

/// The longest answer to the confirmation.
const MAX_ANSWER_BYTES: u64 = 64;

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// The DNS zone of the apps' public domains, on Cloudflare. Not available yet.
    #[arg(long, value_name = "ZONE")]
    domain: Option<String>,
    /// How the admin reaches the panel.
    #[arg(long, value_enum, default_value_t = AdminMode::Tailscale)]
    admin: AdminMode,
    /// Installs without asking for confirmation.
    #[arg(long)]
    yes: bool,
}

impl InstallArgs {
    #[cfg(test)]
    pub fn admin(&self) -> AdminMode {
        self.admin
    }
}

/// How the admin reaches the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum AdminMode {
    /// Through `tailscale serve`, over HTTPS on the tailnet.
    Tailscale,
    /// Through an SSH tunnel to 127.0.0.1.
    SshTunnel,
}

/// What the admin asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub admin: AdminMode,
    pub domain: Option<String>,
}

/// A step of the install. Running it twice is safe: `apply` runs only when `is_done` says the
/// host needs it.
pub trait Step {
    /// What the step does, as the install prints it.
    fn name(&self) -> &str;
    /// Whether the host is already as the step leaves it. Reads the host only, and answers
    /// `false`, not an error, when an earlier step is not done.
    fn is_done(&self, host: &Host) -> anyhow::Result<bool>;
    /// Makes the host as the step leaves it, from any state, a half-done one included.
    fn apply(&self, host: &mut Host) -> anyhow::Result<()>;
}

/// What happened to a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    AlreadyDone,
    Applied,
}

/// Runs `steps` in order, each only if it is not done, and prints each one to `out`. Stops at
/// the first failure.
pub fn run_steps(
    steps: &[Box<dyn Step>],
    host: &mut Host,
    out: &mut dyn Write,
) -> anyhow::Result<Vec<Outcome>> {
    let mut outcomes = Vec::with_capacity(steps.len());
    for step in steps {
        let name = step.name();
        write!(out, "- {name}… ")?;
        out.flush()?;
        let done = step
            .is_done(host)
            .with_context(|| format!("{name}: cannot tell whether it is done"));
        let result = match done {
            Ok(true) => {
                writeln!(out, "already done")?;
                outcomes.push(Outcome::AlreadyDone);
                continue;
            }
            Ok(false) => step.apply(host).with_context(|| format!("{name} failed")),
            Err(error) => Err(error),
        };
        match result {
            Ok(()) => {
                writeln!(out, "done")?;
                outcomes.push(Outcome::Applied);
            }
            Err(error) => {
                writeln!(out, "failed")?;
                return Err(error);
            }
        }
    }
    Ok(outcomes)
}

/// The names of the steps that are not done.
fn pending(steps: &[Box<dyn Step>], host: &Host) -> anyhow::Result<Vec<String>> {
    let mut names = Vec::new();
    for step in steps {
        if !step.is_done(host)? {
            names.push(step.name().to_owned());
        }
    }
    Ok(names)
}

/// Installs Brume on this server.
pub fn run(args: &InstallArgs) -> anyhow::Result<()> {
    let options = Options {
        admin: args.admin,
        domain: args.domain.clone(),
    };
    let mut host = Host::real();
    let platform = preflight::check(&host)?;
    let steps = steps::steps(&options, &platform)?;
    let pending = pending(&steps, &host)?;
    if pending.is_empty() {
        println!("This server is already a Brume host. Checking each step:");
    } else {
        println!(
            "brume install turns this server into a Brume host. Its firewall drops every \
             incoming connection except SSH (port {}) and Tailscale. It will:",
            nftables::SSH_PORT
        );
        for name in &pending {
            println!("  {name}");
        }
        if !args.yes {
            confirm()?;
        }
        println!();
    }
    let mut out = std::io::stdout();
    if let Err(error) = run_steps(&steps, &mut host, &mut out) {
        bail!(
            "{error:#}\n\nOnce the cause is fixed, run the same command again: the steps already \
             done are skipped."
        );
    }
    finish(&host)
}

/// Asks the admin to go on. Any answer but yes stops the install before any change.
fn confirm() -> anyhow::Result<()> {
    print!("Continue? [y/N] ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin()
        .lock()
        .take(MAX_ANSWER_BYTES)
        .read_line(&mut answer)
        .context("cannot read the answer")?;
    match answer.trim() {
        "y" | "Y" | "yes" => Ok(()),
        "" if answer.is_empty() => {
            bail!("No answer: nothing was changed. Add --yes to install without a question.")
        }
        _ => bail!("Nothing was changed."),
    }
}

/// Waits for the engine, then prints a login link, and warns about any unit that does not run.
fn finish(host: &Host) -> anyhow::Result<()> {
    let file = ApiFileConfig::load(Path::new(SYSTEM_CONFIG_PATH))?;
    let paths = Paths::system();
    let deadline = Instant::now() + LINK_TIMEOUT;
    let link = loop {
        match login::request_link(&paths) {
            Ok(link) => break link,
            Err(error) if Instant::now() >= deadline => {
                return Err(error.context(
                    "The engine gave no login link within a minute. Check its log: journalctl \
                     -u brume-engine",
                ));
            }
            Err(_) => std::thread::sleep(LINK_POLL),
        }
    };
    println!();
    for unit in systemd::checked_units() {
        if !systemd::is_active(host, &unit)? {
            println!("Warning: {unit} is not running. Check its log: journalctl -u {unit}");
        }
    }
    println!("Brume is installed.");
    login::print_link(&file, &link, &login::ssh_destination_from_env());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::host::fake::FakeSystem;
    use super::host::{Cmd, DirMeta, Host, Output};
    use super::preflight::{Distro, Platform};
    use super::steps::steps;
    use super::systemd::{self, Service};
    use super::{AdminMode, Options, Outcome, docker, nftables, run_steps, users};

    fn platform() -> Platform {
        Platform {
            distro: Distro::Ubuntu,
            codename: "noble".to_owned(),
            arch: "arm64",
        }
    }

    fn ssh_tunnel() -> Options {
        Options {
            admin: AdminMode::SshTunnel,
            domain: None,
        }
    }

    const DOCKER: &str = "installed 5:29.0.1-1~ubuntu.24.04~noble";

    /// The answers of an installed host to the queries of the steps.
    fn answer_installed(fake: &FakeSystem) {
        for package in ["docker-ce", "docker-ce-cli", "containerd.io"] {
            fake.answer(&docker::package_query(package), Output::ok(DOCKER));
        }
        fake.answer(
            &docker::security_options(),
            Output::ok("[\"name=seccomp,profile=builtin\",\"name=userns\"]\n"),
        );
        fake.answer(&nftables::list_table(), Output::ok("table inet brume {}\n"));
        fake.answer(&users::group_query("brume"), Output::ok("brume:x:996:\n"));
        for service in Service::ALL {
            let user = service.user();
            fake.answer(&users::user_query(user), Output::ok(user));
            let groups = if service == Service::Agent {
                "brume-agent brume docker"
            } else {
                "brume-x brume"
            };
            fake.answer(&users::groups_query(user), Output::ok(groups));
        }
        for unit in systemd::checked_units() {
            fake.answer(&systemd::is_enabled_query(&unit), Output::ok("enabled\n"));
            fake.answer(&systemd::is_active_query(&unit), Output::ok("active\n"));
        }
    }

    /// The directories that `systemd-tmpfiles` creates: path, owner, group, mode.
    const DIRS: [(&str, &str, &str, u32); 7] = [
        ("/run/brume", "root", "brume", 0o3770),
        ("/var/lib/brume", "root", "root", 0o755),
        (
            "/var/lib/brume/engine",
            "brume-engine",
            "brume-engine",
            0o700,
        ),
        (
            "/var/lib/brume/bucket",
            "brume-engine",
            "brume-engine",
            0o700,
        ),
        (
            "/var/lib/brume/telemetry",
            "brume-telemetry",
            "brume-telemetry",
            0o700,
        ),
        ("/var/lib/brume/agent", "brume-agent", "brume-agent", 0o755),
        (
            "/var/lib/brume/agent/traefik",
            "brume-agent",
            "brume-agent",
            0o755,
        ),
    ];

    /// A host where a first install went through: the files that the steps wrote, the
    /// directories and the answers of an installed host, and no change recorded yet.
    fn installed() -> FakeSystem {
        let fake = FakeSystem::default();
        answer_installed(&fake);
        let steps = steps(&ssh_tunnel(), &platform()).expect("steps");
        let mut host = Host::new(Box::new(fake.clone()));
        run_steps(&steps, &mut host, &mut Vec::new()).expect("installs");
        for (path, user, group, mode) in DIRS {
            fake.put_dir(path, DirMeta::new(user, group, mode));
        }
        fake.put_path("/usr/sbin/nft");
        fake.clear_changes();
        fake
    }

    #[test]
    fn rerun_skips_done_steps() {
        let fake = installed();
        let all = steps(&ssh_tunnel(), &platform()).expect("steps");
        let mut host = Host::new(Box::new(fake.clone()));
        let mut out = Vec::new();
        let outcomes = run_steps(&all, &mut host, &mut out).expect("runs");
        assert_eq!(outcomes, vec![Outcome::AlreadyDone; all.len()]);
        assert_eq!(fake.changes(), Vec::<String>::new());
        let out = String::from_utf8(out).expect("UTF-8");
        assert_eq!(out.lines().count(), all.len());
        assert!(
            out.lines().all(|line| line.ends_with("… already done")),
            "{out}"
        );
    }

    #[test]
    fn a_half_done_install_does_only_what_is_missing() {
        let fake = installed();
        // Stopped after Docker: no users, no units, nothing started.
        fake.answer(&users::group_query("brume"), Output::failed(2));
        for service in Service::ALL {
            fake.answer(&users::user_query(service.user()), Output::failed(2));
            fake.answer(
                &systemd::is_active_query(&service.unit()),
                Output::ok("inactive\n"),
            );
        }
        fake.put_file("/etc/systemd/system/brume-agent.service", "an older unit\n");
        let all = steps(&ssh_tunnel(), &platform()).expect("steps");
        let mut host = Host::new(Box::new(fake.clone()));
        let outcomes = run_steps(&all, &mut host, &mut Vec::new()).expect("runs");
        let applied: Vec<&str> = all
            .iter()
            .zip(&outcomes)
            .filter(|(_, outcome)| **outcome == Outcome::Applied)
            .map(|(step, _)| step.name())
            .collect();
        assert_eq!(
            applied,
            [
                "Create the system users of the services",
                "Write the systemd units of the services",
                "Start the services",
            ]
        );
        let changes = fake.changes();
        assert_eq!(changes[0], "groupadd --system brume");
        assert!(
            changes.contains(
                &"useradd --system --user-group --no-create-home --home-dir /nonexistent --shell \
              /usr/sbin/nologin --groups brume,docker brume-agent"
                    .to_owned()
            )
        );
        assert!(changes.contains(&"write /etc/systemd/system/brume-agent.service".to_owned()));
        assert!(!changes.contains(&"write /etc/systemd/system/brume-api.service".to_owned()));
        assert!(changes.contains(&"systemctl try-restart brume-agent.service".to_owned()));
        assert_eq!(
            changes.last().map(String::as_str),
            Some(
                "systemctl enable --now brume-telemetry.service brume-engine.service \
                 brume-agent.service brume-api.service"
            )
        );
        // Nothing touches the engine's data, its master key included.
        assert!(
            changes
                .iter()
                .all(|change| !change.contains("/var/lib/brume"))
        );
    }

    #[test]
    fn a_fresh_host_gets_the_firewall_before_docker() {
        let fake = FakeSystem::default();
        let install = Cmd::new(
            "apt-get",
            &[
                "-y",
                "-q",
                "-o",
                "DPkg::Lock::Timeout=600",
                "install",
                "--no-install-recommends",
                "docker-ce",
                "docker-ce-cli",
                "containerd.io",
            ],
        );
        for package in ["docker-ce", "docker-ce-cli", "containerd.io"] {
            fake.answer_after(
                &install,
                &docker::package_query(package),
                Output::ok(DOCKER),
            );
        }
        fake.answer_after(
            &Cmd::new("systemctl", &["restart", "docker.service"]),
            &docker::security_options(),
            Output::ok("[\"name=userns\"]"),
        );
        let all = steps(&ssh_tunnel(), &platform()).expect("steps");
        let mut host = Host::new(Box::new(fake.clone()));
        let outcomes = run_steps(&all, &mut host, &mut Vec::new()).expect("installs");
        assert_eq!(outcomes, vec![Outcome::Applied; all.len()]);
        let changes = fake.changes();
        let at = |change: &str| {
            changes
                .iter()
                .position(|line| line == change)
                .unwrap_or_else(|| panic!("{change} in {changes:#?}"))
        };
        let firewall = at("systemctl reload-or-restart brume-nftables.service");
        let daemon_json = at("write /etc/docker/daemon.json");
        let docker = at(&install.to_string());
        assert!(firewall < daemon_json && daemon_json < docker);
        assert!(at("write /etc/apt/preferences.d/docker") > docker);
        assert!(
            at(
                "useradd --system --user-group --no-create-home --home-dir /nonexistent \
                    --shell /usr/sbin/nologin --groups brume brume-api"
            ) > docker
        );
        assert!(at("systemd-tmpfiles --create /etc/tmpfiles.d/brume.conf") > firewall);
        assert_eq!(
            fake.file("/etc/apt/preferences.d/docker").as_deref(),
            Some(
                "# Written by `brume install`: Docker gets the fixes of the major version Brume \
                 was installed with.\nPackage: docker-ce docker-ce-cli\nPin: version 5:29.*\n\
                 Pin-Priority: 990\n"
            )
        );
        assert_eq!(
            fake.file("/etc/brume/brume.nft").as_deref(),
            Some(nftables::RULES)
        );
    }

    #[test]
    fn an_old_docker_stops_the_install() {
        let fake = FakeSystem::default();
        for package in ["docker-ce", "docker-ce-cli", "containerd.io"] {
            fake.answer(
                &docker::package_query(package),
                Output::ok("installed 5:28.5.1-1~ubuntu.24.04~noble"),
            );
        }
        let all = steps(&ssh_tunnel(), &platform()).expect("steps");
        let mut host = Host::new(Box::new(fake.clone()));
        let error = run_steps(&all, &mut host, &mut Vec::new()).expect_err("too old");
        assert!(
            format!("{error:#}").contains("Docker 5:28.5.1-1~ubuntu.24.04~noble"),
            "{error:#}"
        );
        assert!(
            !fake
                .changes()
                .iter()
                .any(|change| change.starts_with("useradd"))
        );
    }

    #[test]
    fn tailscale_and_domains_come_later() {
        for options in [
            Options {
                admin: AdminMode::Tailscale,
                domain: None,
            },
            Options {
                admin: AdminMode::SshTunnel,
                domain: Some("example.com".to_owned()),
            },
        ] {
            let error = steps(&options, &platform()).err().expect("not yet");
            assert!(error.to_string().contains("ssh-tunnel"), "{error}");
        }
    }
}
