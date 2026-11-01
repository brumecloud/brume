//! `brume install`, run as root on a fresh server: turns it into a Brume host. Brume's
//! firewall, Docker with user namespaces, Tailscale with the panel on the tailnet, one system
//! user per service, hardened systemd units, and with `--domain`, a Cloudflare tunnel for the
//! apps' public domains. Then a login link to the panel.
//!
//! Each step checks whether it is done before doing anything: running the install again, after
//! a failure or on an installed host, does only what is missing, then prints a new login link.
//! The engine's master key is never touched: the engine creates it at its first start. Nor is
//! the Cloudflare API token ever written: it stays in this process's memory.

mod api_config;
mod cloudflare;
mod docker;
mod host;
mod nftables;
mod preflight;
mod steps;
mod systemd;
mod tailscale;
mod users;
mod wait;

use std::io::{BufRead as _, Read as _, Write};
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use brume_api::{ApiFileConfig, SYSTEM_CONFIG_PATH};
use brume_runtime::paths::Paths;
use clap::{Args, ValueEnum};

use self::cloudflare::{Cloudflare, Site};
use self::host::Host;
use self::steps::Context;
use self::wait::Console;
use crate::{admin, login};

/// The longest wait for the engine's first login link, after the services start.
const LINK_TIMEOUT: Duration = Duration::from_secs(60);

/// How often the engine is asked while it starts.
const LINK_POLL: Duration = Duration::from_millis(500);

/// The longest answer to the confirmation.
const MAX_ANSWER_BYTES: u64 = 64;

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// The DNS zone of the apps' public domains, on Cloudflare, such as example.com. The
    /// Cloudflare API token comes from CLOUDFLARE_API_TOKEN, or is asked for: sudo drops the
    /// variable unless run as `sudo --preserve-env=CLOUDFLARE_API_TOKEN`.
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
    // Before any change: a token that lacks a permission, or a wildcard that points elsewhere,
    // stops the install here.
    let cloudflare = match &options.domain {
        Some(domain) => Some(check_cloudflare(&host, domain)?),
        None => None,
    };
    let context = Context {
        console: Console::real(),
        cloudflare,
    };
    let steps = steps::steps(&options, &platform, &context)?;
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
        bail!(failure_message(&error));
    }
    finish(
        &host,
        context
            .cloudflare
            .as_ref()
            .map(|cloudflare| &*cloudflare.site),
    )
}

/// Reads the Cloudflare API token, and checks that it can set up `domain`.
fn check_cloudflare(host: &Host, domain: &str) -> anyhow::Result<Cloudflare> {
    let zone = cloudflare::parse_zone(domain)?;
    let token = cloudflare::read_token(&zone)?;
    let client = cloudflare::Client::new(token)?;
    println!("Checking the Cloudflare API token on {zone}…");
    let site = cloudflare::check(host, &client, &zone)?;
    Ok(Cloudflare {
        client: Rc::new(client),
        site: Rc::new(site),
    })
}

/// What the install prints when a step fails: the cause, then how to go on.
fn failure_message(error: &anyhow::Error) -> String {
    let next = if wait::timed_out(error) {
        "Run the same command again to resume: it skips the steps already done, prints the link \
         again and waits."
    } else {
        "Once the cause is fixed, run the same command again: the steps already done are \
         skipped."
    };
    format!("{error:#}\n\n{next}")
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
fn finish(host: &Host, site: Option<&Site>) -> anyhow::Result<()> {
    let file = ApiFileConfig::load(Path::new(SYSTEM_CONFIG_PATH))?;
    let paths = Paths::system();
    let deadline = Instant::now() + LINK_TIMEOUT;
    let link = loop {
        match login::request_link(&paths) {
            Ok(link) => break link,
            Err(error) if admin::is_refusal(&error) => return Err(error),
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
    for note in final_notes(&file, site) {
        println!("{note}");
    }
    Ok(())
}

/// What the install prints after the login link: on a tailnet, how to keep the panel past the
/// expiry of the node's key; with a domain, how the services get public domains.
fn final_notes(file: &ApiFileConfig, site: Option<&Site>) -> Vec<String> {
    let mut notes = Vec::new();
    if let Some(name) = file.hosts.first() {
        notes.push(tailscale::key_expiry_note(name));
    }
    if let Some(site) = site {
        notes.push(domains_message(site));
    }
    notes
}

/// How the services get public domains, and whether Cloudflare serves the zone yet.
fn domains_message(site: &Site) -> String {
    let zone = &site.zone;
    let mut text = format!(
        "Public domains: in the panel, a service's settings give it <name>.{zone}, served over \
         HTTPS through the tunnel {}.",
        site.tunnel_name
    );
    if !site.zone_active() {
        text.push_str(&format!(
            "\nWarning: {zone} is {} on Cloudflare. Its domains answer once Cloudflare \
             activates it: set Cloudflare's name servers at the registrar of {zone}.",
            site.zone_status
        ));
    }
    text
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::time::Duration;

    use brume_api::ApiFileConfig;
    use secrecy::SecretString;

    use super::cloudflare::fake::{API_TOKEN, FakeCloudflare, State};
    use super::cloudflare::{self, Cloudflare};
    use super::host::fake::FakeSystem;
    use super::host::{Cmd, DirMeta, Host, Output};
    use super::preflight::{Distro, Platform};
    use super::steps::{Context, steps};
    use super::systemd::{self, Service};
    use super::tailscale::fake::{SERVING, status_json};
    use super::tailscale::{Status, serve_command, serve_status_query, status_query, up_command};
    use super::wait::fake::FakeConsole;
    use super::{
        AdminMode, Options, Outcome, docker, domains_message, failure_message, final_notes,
        nftables, run_steps, users, wait,
    };

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

    /// No Cloudflare, and a console of its own.
    fn context() -> Context {
        Context {
            console: FakeConsole::new().console,
            cloudflare: None,
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
    const DIRS: [(&str, &str, &str, u32); 9] = [
        ("/run/brume", "root", "root", 0o755),
        ("/run/brume/engine", "brume-engine", "brume", 0o2750),
        ("/run/brume/telemetry", "brume-telemetry", "brume", 0o2750),
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
        let steps = steps(&ssh_tunnel(), &platform(), &context()).expect("steps");
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
        let all = steps(&ssh_tunnel(), &platform(), &context()).expect("steps");
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
        let all = steps(&ssh_tunnel(), &platform(), &context()).expect("steps");
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
        let all = steps(&ssh_tunnel(), &platform(), &context()).expect("steps");
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
        let all = steps(&ssh_tunnel(), &platform(), &context()).expect("steps");
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

    /// `tailscale status --json` of a node on the tailnet, owned by paul@example.com.
    fn running_json() -> String {
        status_json("Running", "paul@example.com", &[], true)
    }

    /// The answers of a host where Tailscale runs, joined and serving the panel.
    fn answer_tailscale_up(fake: &FakeSystem) {
        fake.answer(
            &docker::package_query("tailscale"),
            Output::ok("installed 1.90.6"),
        );
        for query in [
            systemd::is_enabled_query("tailscaled.service"),
            systemd::is_active_query("tailscaled.service"),
        ] {
            let state = if query.to_string().contains("enabled") {
                "enabled\n"
            } else {
                "active\n"
            };
            fake.answer(&query, Output::ok(state));
        }
        fake.answer(&status_query(), Output::ok(&running_json()));
        fake.answer(&serve_status_query(), Output::ok(SERVING));
    }

    fn tailscale() -> Options {
        Options {
            admin: AdminMode::Tailscale,
            domain: None,
        }
    }

    /// The step named `name`, alone.
    fn only(all: Vec<Box<dyn super::Step>>, name: &str) -> Vec<Box<dyn super::Step>> {
        let kept: Vec<_> = all.into_iter().filter(|step| step.name() == name).collect();
        assert_eq!(kept.len(), 1, "{name}");
        kept
    }

    #[test]
    fn the_tailnet_comes_before_the_services() {
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: None,
        };
        let all = steps(&tailscale(), &platform(), &context).expect("steps");
        let names: Vec<&str> = all.iter().map(|step| step.name()).collect();
        let at = |name: &str| {
            names
                .iter()
                .position(|found| *found == name)
                .unwrap_or_else(|| panic!("{name} in {names:#?}"))
        };
        assert!(at("Start Docker, with user namespaces") < at("Add Tailscale's apt repository"));
        assert!(
            at("Join your tailnet: open the link printed here")
                < at("Serve the panel on your tailnet over HTTPS (tailscale serve)")
        );
        assert!(
            at("Serve the panel on your tailnet over HTTPS (tailscale serve)")
                < at("Create the system users of the services")
        );
        assert!(at("Write /etc/brume/api.toml") < at("Start the services"));
    }

    #[test]
    fn a_tailscale_install_waits_for_the_login_then_serves_the_panel() {
        let fake = FakeSystem::default();
        // Not logged in: `tailscale up` gives the link, then the admin clicks it.
        let needs_login = || Output::ok(&status_json("NeedsLogin", "", &[], true));
        fake.answer(&status_query(), needs_login());
        fake.answer(
            &up_command(),
            Output {
                code: Some(1),
                stdout: "{\n\t\"AuthURL\": \"https://login.tailscale.com/a/1a2b3c4d5e6f\",\n\t\
                         \"BackendState\": \"NeedsLogin\"\n}\n"
                    .to_owned(),
                stderr: "timeout waiting for Tailscale service to enter a Running state\n"
                    .to_owned(),
            },
        );
        fake.answer_after_each(
            &up_command(),
            &status_query(),
            vec![needs_login(), needs_login(), Output::ok(&running_json())],
        );
        fake.answer_after(
            &serve_command("60"),
            &serve_status_query(),
            Output::ok(SERVING),
        );
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: None,
        };
        let all = steps(&tailscale(), &platform(), &context).expect("steps");
        let all: Vec<_> = all
            .into_iter()
            .filter(|step| {
                [
                    "Join your tailnet: open the link printed here",
                    "Serve the panel on your tailnet over HTTPS (tailscale serve)",
                    "Write /etc/brume/api.toml",
                ]
                .contains(&step.name())
            })
            .collect();
        let mut host = Host::new(Box::new(fake.clone()));
        let outcomes = run_steps(&all, &mut host, &mut Vec::new()).expect("installs");
        assert_eq!(outcomes, vec![Outcome::Applied; 3]);
        assert_eq!(
            fake.changes(),
            [
                "timeout 30 tailscale up --ssh --hostname=brume --json --timeout=10s",
                "timeout 60 tailscale serve --bg --https=443 http://127.0.0.1:7878",
                "write /etc/brume/api.toml",
            ]
        );
        let printed = console.printed();
        assert_eq!(
            printed,
            "  To put this server on your tailnet, open this link and log in to Tailscale:\n    \
             https://login.tailscale.com/a/1a2b3c4d5e6f\n  Waiting for the Tailscale login, 15 \
             minutes at most…\n"
        );
        let file = fake.file("/etc/brume/api.toml").expect("written");
        assert!(
            file.contains("hosts = [\"brume.tail1234.ts.net\"]"),
            "{file}"
        );
        assert!(
            file.contains("tailscale_logins = [\"paul@example.com\"]"),
            "{file}"
        );
    }

    #[test]
    fn login_wait_times_out_with_resume_message() {
        let fake = FakeSystem::default();
        fake.answer(
            &status_query(),
            Output::ok(&status_json("NeedsLogin", "", &[], true)),
        );
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: None,
        };
        let all = only(
            steps(&tailscale(), &platform(), &context).expect("steps"),
            "Join your tailnet: open the link printed here",
        );
        let mut host = Host::new(Box::new(fake.clone()));
        let mut out = Vec::new();
        let error = run_steps(&all, &mut host, &mut out).expect_err("nobody logs in");
        let message = failure_message(&error);
        assert_eq!(
            message,
            "Join your tailnet: open the link printed here failed: Gave up waiting for the \
             Tailscale login after 15 minutes (https://login.tailscale.com/a/1a2b3c4d5e6f).\n\n\
             Run the same command again to resume: it skips the steps already done, prints the \
             link again and waits."
        );
        // The link, at once and every two minutes.
        let printed = console.printed();
        assert_eq!(
            printed
                .matches("https://login.tailscale.com/a/1a2b3c4d5e6f")
                .count(),
            8,
            "{printed}"
        );
        assert!(printed.contains("  Still waiting for the Tailscale login, 13 min left: https://"));
        let waited = console.elapsed().as_secs();
        assert!((900..905).contains(&waited), "{waited}");
        // An error that is not a wait keeps the usual advice.
        let other = failure_message(&anyhow::anyhow!("apt failed"));
        assert!(other.ends_with("Once the cause is fixed, run the same command again: the steps already done are skipped."));
    }

    #[test]
    fn https_off_prints_the_link_that_turns_it_on() {
        let fake = FakeSystem::default();
        let https_off = || Output::ok(&status_json("Running", "paul@example.com", &[], false));
        fake.answer(&status_query(), https_off());
        fake.answer(
            &serve_command("15"),
            Output {
                code: Some(124),
                stdout: "\nServe is not enabled on your tailnet.\nTo enable, visit:\n\n         \
                         https://login.tailscale.com/f/serve?node=nTSv3h5CNTRL\n\n"
                    .to_owned(),
                stderr: String::new(),
            },
        );
        fake.answer_after_each(
            &serve_command("15"),
            &status_query(),
            vec![https_off(), https_off(), Output::ok(&running_json())],
        );
        fake.answer_after(
            &serve_command("60"),
            &serve_status_query(),
            Output::ok(SERVING),
        );
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: None,
        };
        let all = only(
            steps(&tailscale(), &platform(), &context).expect("steps"),
            "Serve the panel on your tailnet over HTTPS (tailscale serve)",
        );
        let mut host = Host::new(Box::new(fake.clone()));
        run_steps(&all, &mut host, &mut Vec::new()).expect("serves");
        assert_eq!(
            fake.changes(),
            [
                "timeout 15 tailscale serve --bg --https=443 http://127.0.0.1:7878",
                "timeout 60 tailscale serve --bg --https=443 http://127.0.0.1:7878",
            ]
        );
        assert!(
            console.printed().contains(
                "Your tailnet has HTTPS turned off, and the panel needs it. To turn it on, \
                 open:\n    https://login.tailscale.com/f/serve?node=nTSv3h5CNTRL\n"
            ),
            "{}",
            console.printed()
        );
    }

    #[test]
    fn a_second_tailscale_up_gets_the_link_that_the_first_did_not_have() {
        let fake = FakeSystem::default();
        let no_link = status_json("NeedsLogin", "", &[], true)
            .replace("https://login.tailscale.com/a/1a2b3c4d5e6f", "");
        fake.answer(&status_query(), Output::ok(&no_link));
        let timed_out = Output {
            code: Some(1),
            stdout: String::new(),
            stderr: "timeout waiting for Tailscale service to enter a Running state\n".to_owned(),
        };
        let with_link = Output {
            code: Some(1),
            stdout: "{\"AuthURL\": \"https://login.tailscale.com/a/second\"}\n".to_owned(),
            stderr: String::new(),
        };
        fake.answer_each(&up_command(), vec![timed_out, with_link]);
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: None,
        };
        let all = only(
            steps(&tailscale(), &platform(), &context).expect("steps"),
            "Join your tailnet: open the link printed here",
        );
        let mut host = Host::new(Box::new(fake.clone()));
        run_steps(&all, &mut host, &mut Vec::new()).expect_err("nobody logs in");
        assert_eq!(
            fake.changes(),
            [
                "timeout 30 tailscale up --ssh --hostname=brume --json --timeout=10s",
                "timeout 30 tailscale up --ssh --hostname=brume --json --timeout=10s",
            ]
        );
        assert!(
            console.printed().starts_with(
                "  To put this server on your tailnet, open this link and log in to \
                              Tailscale:\n    https://login.tailscale.com/a/second\n"
            ),
            "{}",
            console.printed()
        );
    }

    /// The step named `name` of a Tailscale install, on `fake`, and what it printed.
    fn run_tailscale_step(
        fake: &FakeSystem,
        name: &str,
    ) -> (anyhow::Result<Vec<Outcome>>, FakeConsole) {
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: None,
        };
        let all = only(
            steps(&tailscale(), &platform(), &context).expect("steps"),
            name,
        );
        let mut host = Host::new(Box::new(fake.clone()));
        let outcomes = run_steps(&all, &mut host, &mut Vec::new());
        (outcomes, console)
    }

    const JOIN: &str = "Join your tailnet: open the link printed here";
    const SERVE: &str = "Serve the panel on your tailnet over HTTPS (tailscale serve)";

    /// `tailscale up` that handed over after its `--timeout`, with the login link.
    fn handed_over() -> Output {
        Output {
            code: Some(1),
            stdout: "{\n\t\"AuthURL\": \"https://login.tailscale.com/a/1a2b3c4d5e6f\",\n\t\"QR\": \
                     \"data:image/png;base64,iVBORw0KGgo=\",\n\t\"BackendState\": \"NeedsLogin\"\n}\n"
                .to_owned(),
            stderr: "timeout waiting for Tailscale service to enter a Running state; check health \
                     with \"tailscale status\"\n"
                .to_owned(),
        }
    }

    #[test]
    fn serve_that_exits_at_once_still_waits_with_its_link() {
        let fake = FakeSystem::default();
        let https_off = || Output::ok(&status_json("Running", "paul@example.com", &[], false));
        fake.answer(&status_query(), https_off());
        // Tailscale tells `tailscale serve` not to wait: it prints the link and exits 0.
        fake.answer(
            &serve_command("15"),
            Output::ok(
                "\nServe is not enabled on your tailnet.\nTo enable, visit:\n\n         \
                 https://login.tailscale.com/f/serve?node=nTSv3h5CNTRL\n\n",
            ),
        );
        fake.answer_after_each(
            &serve_command("15"),
            &status_query(),
            vec![https_off(), https_off(), Output::ok(&running_json())],
        );
        fake.answer_after(
            &serve_command("60"),
            &serve_status_query(),
            Output::ok(SERVING),
        );
        let (outcomes, console) = run_tailscale_step(&fake, SERVE);
        outcomes.expect("serves");
        assert!(
            console.printed().contains(
                "To turn it on, open:\n    https://login.tailscale.com/f/serve?node=nTSv3h5CNTRL\n"
            ),
            "{}",
            console.printed()
        );
        assert!(console.elapsed() >= Duration::from_secs(4));
        assert_eq!(
            fake.changes(),
            [
                "timeout 15 tailscale serve --bg --https=443 http://127.0.0.1:7878",
                "timeout 60 tailscale serve --bg --https=443 http://127.0.0.1:7878",
            ]
        );

        // A failure without a link, and HTTPS still off: Tailscale's error, at once.
        let fake = FakeSystem::default();
        fake.answer(&status_query(), https_off());
        fake.answer(
            &serve_command("15"),
            Output {
                code: Some(1),
                stdout: String::new(),
                stderr: "error getting serve config: 403 Forbidden\n".to_owned(),
            },
        );
        let (outcomes, console) = run_tailscale_step(&fake, SERVE);
        let error = format!("{:#}", outcomes.expect_err("fails"));
        assert!(
            error.contains("`tailscale serve` failed:\nerror getting serve config: 403 Forbidden"),
            "{error}"
        );
        assert_eq!(console.elapsed(), Duration::ZERO);
    }

    #[test]
    fn a_failed_tailscale_up_stops_at_once_with_its_error() {
        let no_link = status_json("NeedsLogin", "", &[], true)
            .replace("https://login.tailscale.com/a/1a2b3c4d5e6f", "");
        for (output, shown) in [
            (
                Output {
                    code: Some(1),
                    stdout: String::new(),
                    stderr: "backend error: invalid key: unable to validate API key\n".to_owned(),
                },
                "`tailscale up` failed, and Tailscale is not up:\nbackend error: invalid key: \
                 unable to validate API key",
            ),
            (
                Output::failed(124),
                "`tailscale up` did not finish within 30 seconds, and Tailscale is not up",
            ),
        ] {
            let fake = FakeSystem::default();
            fake.answer(&status_query(), Output::ok(&no_link));
            fake.answer(&up_command(), output);
            let (outcomes, console) = run_tailscale_step(&fake, JOIN);
            let error = outcomes.expect_err("fails");
            assert!(!wait::timed_out(&error), "{error:#}");
            assert!(format!("{error:#}").contains(shown), "{error:#}");
            assert_eq!(console.elapsed(), Duration::ZERO);
            assert_eq!(console.printed(), "");
            assert_eq!(
                fake.changes(),
                ["timeout 30 tailscale up --ssh --hostname=brume --json --timeout=10s"]
            );
        }
    }

    #[test]
    fn a_failed_status_poll_does_not_stop_the_login_wait() {
        let fake = FakeSystem::default();
        let needs_login = || Output::ok(&status_json("NeedsLogin", "", &[], true));
        fake.answer(&status_query(), needs_login());
        fake.answer(&up_command(), handed_over());
        fake.answer_after_each(
            &up_command(),
            &status_query(),
            vec![
                Output::failed(1),
                Output::ok("not JSON"),
                needs_login(),
                Output::ok(&running_json()),
            ],
        );
        let (outcomes, console) = run_tailscale_step(&fake, JOIN);
        outcomes.expect("joins");
        assert!(
            console
                .printed()
                .contains("https://login.tailscale.com/a/1a2b3c4d5e6f"),
            "{}",
            console.printed()
        );
    }

    #[test]
    fn an_unknown_tailscale_state_is_named() {
        for (status, shown) in [
            (
                Output::ok(&status_json("InUseOtherUser", "", &[], true)),
                "Tailscale stays in the state InUseOtherUser",
            ),
            (
                Output {
                    code: Some(1),
                    stdout: String::new(),
                    stderr: "failed to connect to local tailscaled; it doesn't appear to be \
                             running\n"
                        .to_owned(),
                },
                "`tailscale status --json` failed 5 times in a row: failed to connect to local \
                 tailscaled",
            ),
        ] {
            let fake = FakeSystem::default();
            fake.answer(
                &status_query(),
                Output::ok(&status_json("NeedsLogin", "", &[], true)),
            );
            fake.answer(&up_command(), handed_over());
            fake.answer_after(&up_command(), &status_query(), status);
            let (outcomes, console) = run_tailscale_step(&fake, JOIN);
            let error = outcomes.expect_err("stops");
            assert!(!wait::timed_out(&error), "{error:#}");
            let message = format!("{error:#}");
            assert!(message.contains(shown), "{message}");
            assert!(!message.contains("base64"), "{message}");
            // A few polls, not the whole wait.
            assert!(console.elapsed() < Duration::from_secs(60));
        }
    }

    #[test]
    fn the_closing_notes_ask_to_disable_key_expiry_on_the_tailnet() {
        let file = Status::parse(&running_json())
            .expect("parses")
            .panel_file()
            .expect("a file");
        let notes = final_notes(&file, None);
        assert_eq!(
            notes,
            [
                "Disable key expiry for brume in Tailscale's admin console (Machines → brume → … → \
                 Disable key expiry, https://login.tailscale.com/admin/machines): its key \
                 expires after 180 days otherwise, and the panel with it."
            ]
        );
        // An SSH tunnel does not go through Tailscale.
        let ssh = ApiFileConfig {
            hosts: Vec::new(),
            tailscale_logins: Vec::new(),
            public_url: String::new(),
            ..file
        };
        assert_eq!(final_notes(&ssh, None), Vec::<String>::new());
    }

    #[test]
    fn a_tagged_node_is_refused_before_the_panel_is_served() {
        let fake = FakeSystem::default();
        answer_tailscale_up(&fake);
        fake.answer(
            &status_query(),
            Output::ok(&status_json(
                "Running",
                "paul@example.com",
                &["tag:server"],
                true,
            )),
        );
        fake.answer(&serve_status_query(), Output::failed(1));
        let all = steps(&tailscale(), &platform(), &context()).expect("steps");
        let all: Vec<_> = all
            .into_iter()
            .filter(|step| step.name().contains("tailnet"))
            .collect();
        let mut host = Host::new(Box::new(fake.clone()));
        let error = run_steps(&all, &mut host, &mut Vec::new()).expect_err("tagged");
        assert!(format!("{error:#}").contains("tagged device"), "{error:#}");
        assert_eq!(fake.changes(), Vec::<String>::new());
    }

    #[test]
    fn rerun_with_tailscale_and_cloudflare_changes_nothing() {
        // A host where a first install with Tailscale and --domain went through.
        let fake = installed();
        answer_tailscale_up(&fake);
        fake.put_file("/proc/sys/kernel/hostname", "web-1\n");
        fake.put_file("/etc/machine-id", "4c1f0e6b2a9d4e8f9b7a6c5d4e3f2a1b\n");
        let host = Host::new(Box::new(fake.clone()));
        let tunnel_name = cloudflare::tunnel_name(&host).expect("a name");
        let mut state = State::new("example.com");
        let tunnel = state.add_tunnel(&tunnel_name);
        state.add_record(
            "CNAME",
            "*.example.com",
            &format!("{tunnel}.cfargotunnel.com"),
            true,
        );
        "on".clone_into(&mut state.always_use_https);
        let api = FakeCloudflare::start(state);
        let client = cloudflare::Client::local(api.base(), SecretString::from(API_TOKEN));
        let zone = cloudflare::parse_zone("example.com").expect("a zone");
        let site = cloudflare::check(&host, &client, &zone).expect("passes");
        fake.start_engine();
        let mut engine_host = Host::new(Box::new(fake.clone()));
        engine_host
            .admin_change(&brume_proto::admin::AdminRequest::ConfigureIngress {
                zone: "example.com".to_owned(),
                tunnel_id: tunnel.clone(),
                tunnel_token: SecretString::from("eyJ0IjoiInt"),
            })
            .expect("configured");
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: Some(Cloudflare {
                client: Rc::new(client),
                site: Rc::new(site),
            }),
        };
        let options = Options {
            admin: AdminMode::Tailscale,
            domain: Some("example.com".to_owned()),
        };
        let all = steps(&options, &platform(), &context).expect("steps");
        assert_eq!(all.len(), 21);
        // The first install wrote what Tailscale adds: its repository and the tailnet's
        // api.toml.
        run_steps(&all, &mut engine_host, &mut Vec::new()).expect("installs");
        fake.clear_changes();
        api.state().requests.clear();

        let mut out = Vec::new();
        let outcomes = run_steps(&all, &mut engine_host, &mut out).expect("runs");
        assert_eq!(outcomes, vec![Outcome::AlreadyDone; all.len()]);
        assert_eq!(fake.changes(), Vec::<String>::new());
        assert_eq!(api.state().writes(), Vec::<String>::new());
        assert_eq!(console.printed(), "");
        let out = String::from_utf8(out).expect("UTF-8");
        assert!(
            out.lines().all(|line| line.ends_with("… already done")),
            "{out}"
        );
    }

    #[test]
    fn a_fresh_host_with_a_domain_hands_the_tunnel_over_once_the_services_run() {
        let fake = FakeSystem::default();
        // Docker installs, Tailscale is up already.
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
        fake.answer(&status_query(), Output::ok(&running_json()));
        fake.answer_after(
            &serve_command("60"),
            &serve_status_query(),
            Output::ok(SERVING),
        );
        fake.put_file("/proc/sys/kernel/hostname", "web-1\n");
        fake.put_file("/etc/machine-id", "4c1f0e6b2a9d4e8f9b7a6c5d4e3f2a1b\n");
        fake.start_engine();
        let api = FakeCloudflare::start(State::new("example.com"));
        let client = cloudflare::Client::local(api.base(), SecretString::from(API_TOKEN));
        let zone = cloudflare::parse_zone("example.com").expect("a zone");
        let host = Host::new(Box::new(fake.clone()));
        let site = cloudflare::check(&host, &client, &zone).expect("passes");
        let console = FakeConsole::new();
        let context = Context {
            console: Rc::clone(&console.console),
            cloudflare: Some(Cloudflare {
                client: Rc::new(client),
                site: Rc::new(site.clone()),
            }),
        };
        let options = Options {
            admin: AdminMode::Tailscale,
            domain: Some("example.com".to_owned()),
        };
        let all = steps(&options, &platform(), &context).expect("steps");
        let mut host = Host::new(Box::new(fake.clone()));
        let mut out = Vec::new();
        run_steps(&all, &mut host, &mut out).expect("installs");
        let changes = fake.changes();
        let at = |prefix: &str| {
            changes
                .iter()
                .position(|line| line.starts_with(prefix))
                .unwrap_or_else(|| panic!("{prefix} in {changes:#?}"))
        };
        assert!(at("timeout 60 tailscale serve") < at("useradd"));
        assert!(at("systemctl enable --now brume-telemetry") < at("admin ConfigureIngress"));
        assert!(
            changes
                .last()
                .is_some_and(|line| line.starts_with("admin ConfigureIngress example.com ")),
            "{changes:#?}"
        );
        // Neither token reaches the host: no file, no command, no output.
        let tunnel_token = api.state().tunnels[0].token.clone();
        let out = String::from_utf8(out).expect("UTF-8");
        for text in [changes.join("\n"), out, console.printed()] {
            assert!(!text.contains(API_TOKEN), "{text}");
            assert!(!text.contains(&tunnel_token), "{text}");
        }
        for path in [
            "/etc/brume/api.toml",
            "/etc/systemd/system/brume-engine.service",
        ] {
            let file = fake.file(path).expect("written");
            assert!(!file.contains(API_TOKEN) && !file.contains(&tunnel_token));
        }
        assert_eq!(
            domains_message(&site),
            format!(
                "Public domains: in the panel, a service's settings give it <name>.example.com, \
                 served over HTTPS through the tunnel {}.",
                site.tunnel_name
            )
        );
    }
}
