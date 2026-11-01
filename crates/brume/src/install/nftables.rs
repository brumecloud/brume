//! Brume's firewall: the table `inet brume`, a static file embedded in the binary, loaded by
//! `brume-nftables.service` before Docker starts.
//!
//! On Debian and Ubuntu, `/etc/nftables.conf` starts with `flush ruleset`, and
//! `nftables.service` runs it at each start and reload, and flushes the ruleset when it stops:
//! a `systemctl restart nftables` would delete Brume's table and Docker's rules. A drop-in
//! loads the table again after each of these, then restarts Docker, which adds its rules again;
//! its containers keep running (`live-restore`).

use std::path::Path;

use super::host::{Cmd, Host};
use super::{Step, systemd};

/// The table, without any value from the user or the host.
pub const RULES: &str = include_str!("brume.nft");

/// Where the table's file lives on the host.
pub const RULES_PATH: &str = "/etc/brume/brume.nft";

/// The unit that loads the table.
pub const UNIT: &str = "brume-nftables.service";

/// The drop-in of the distribution's `nftables.service`.
const DROP_IN_PATH: &str = "/etc/systemd/system/nftables.service.d/brume.conf";

/// The SSH port that the input chain lets in.
pub const SSH_PORT: &str = "22";

/// The program that loads the table.
const NFT: &str = "/usr/sbin/nft";

/// `brume-nftables.service`: loads the table before the network is up and before Docker, and
/// again on `systemctl reload`. Stopping it leaves the table: containers may still run.
const UNIT_TEXT: &str = "\
# Written by `brume install`: the next run overwrites any change.
[Unit]
Description=Brume's firewall, the nftables table inet brume
Documentation=https://brume.dev/docs
DefaultDependencies=no
Wants=network-pre.target
After=nftables.service
Before=network-pre.target docker.service shutdown.target
Conflicts=shutdown.target

[Service]
Type=oneshot
RemainAfterExit=yes
StandardInput=null
ExecStart=/usr/sbin/nft -f /etc/brume/brume.nft
ExecReload=/usr/sbin/nft -f /etc/brume/brume.nft
ProtectSystem=full
ProtectHome=yes

[Install]
WantedBy=sysinit.target
";

/// The drop-in of `nftables.service`: after each command that flushes the ruleset, the table
/// comes back, and Docker restarts if it runs. Each command is added to the unit's own, after
/// them. Docker is ordered after `nftables.service`, so its restart waits for the unit's command
/// to end, and `--no-block` is required: a blocking restart from that command would wait for
/// itself. Within one command, the requests merge into one restart. `reset-failed` clears
/// Docker's limit of 3 starts a minute, which a few reloads in a row would reach, leaving
/// Docker stopped and its rules gone.
const DROP_IN_TEXT: &str = "\
# Written by `brume install`: the next run overwrites any change.
# /etc/nftables.conf flushes the ruleset: Brume's table is loaded again after it, and Docker
# restarts to add its rules again. Its containers keep running (live-restore).
[Unit]
Before=docker.service

[Service]
ExecStartPost=/usr/sbin/nft -f /etc/brume/brume.nft
ExecStartPost=-/usr/bin/systemctl reset-failed docker.service
ExecStartPost=-/usr/bin/systemctl --no-block try-restart docker.service
ExecReload=/usr/sbin/nft -f /etc/brume/brume.nft
ExecReload=-/usr/bin/systemctl reset-failed docker.service
ExecReload=-/usr/bin/systemctl --no-block try-restart docker.service
ExecStopPost=/usr/sbin/nft -f /etc/brume/brume.nft
ExecStopPost=-/usr/bin/systemctl reset-failed docker.service
ExecStopPost=-/usr/bin/systemctl --no-block try-restart docker.service
";

/// Installs `nft`, if the distribution did not.
pub struct NftablesPackage;

impl Step for NftablesPackage {
    fn name(&self) -> &str {
        "Install nftables"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(host.exists(Path::new(NFT)))
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.change(&apt_get(&["update"]))?;
        host.change(&apt_get(&[
            "install",
            "--no-install-recommends",
            "nftables",
        ]))?;
        Ok(())
    }
}

/// Writes the table, its unit and the drop-in of `nftables.service`, and loads the table.
pub struct Firewall;

impl Step for Firewall {
    fn name(&self) -> &str {
        "Load Brume's firewall, the nftables table inet brume"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        Ok(host.has_file(Path::new(RULES_PATH), RULES)?
            && host.has_file(&systemd::unit_path(UNIT), UNIT_TEXT)?
            && host.has_file(Path::new(DROP_IN_PATH), DROP_IN_TEXT)?
            && systemd::is_enabled(host, UNIT)?
            && systemd::is_active(host, UNIT)?
            && host.query(&list_table())?.success())
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        host.write(Path::new(RULES_PATH), RULES, 0o644)?;
        host.write(&systemd::unit_path(UNIT), UNIT_TEXT, 0o644)?;
        host.write(Path::new(DROP_IN_PATH), DROP_IN_TEXT, 0o644)?;
        host.change(&systemd::daemon_reload())?;
        host.change(&Cmd::new("systemctl", &["enable", UNIT]))?;
        // Runs `nft -f` again, which replaces the table in one transaction. A reload, when the
        // unit is active, does not restart the agent, which requires the unit.
        host.change(&Cmd::new("systemctl", &["reload-or-restart", UNIT]))?;
        Ok(())
    }
}

/// Succeeds if the table is loaded.
pub fn list_table() -> Cmd {
    Cmd::new(NFT, &["list", "table", "inet", "brume"])
}

/// `apt-get` with the given arguments, without questions, waiting for another apt to finish,
/// such as `unattended-upgrades` on a fresh server.
pub fn apt_get(args: &[&str]) -> Cmd {
    let mut all = vec!["-y", "-q", "-o", "DPkg::Lock::Timeout=600"];
    all.extend_from_slice(args);
    Cmd::new("apt-get", &all)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .env("NEEDRESTART_SUSPEND", "1")
}

#[cfg(test)]
mod tests {
    use super::{DROP_IN_PATH, DROP_IN_TEXT, Firewall, RULES, SSH_PORT, UNIT_TEXT};
    use crate::install::host::fake::FakeSystem;
    use crate::install::host::{Host, Output};
    use crate::install::{Step as _, systemd};

    /// The rules, without comments or blank lines.
    fn rules() -> Vec<&'static str> {
        RULES
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect()
    }

    #[test]
    fn the_input_chain_drops_all_but_ssh_tailscale_and_replies() {
        let rules = rules();
        let input = rules
            .iter()
            .position(|line| line.starts_with("chain input"))
            .expect("an input chain");
        assert_eq!(
            rules[input + 1],
            "type filter hook input priority filter; policy drop;"
        );
        for accepted in [
            "iif \"lo\" accept".to_owned(),
            "ct state vmap { invalid : drop, established : accept, related : accept }".to_owned(),
            "iifname \"tailscale0\" accept".to_owned(),
            "udp dport 41641 accept".to_owned(),
            format!("tcp dport {SSH_PORT} accept"),
        ] {
            assert!(rules.contains(&accepted.as_str()), "{accepted}");
        }
        // Containers are dropped before anything they could reach is accepted.
        let containers = rules
            .iter()
            .position(|line| *line == "iifname \"br-*\" drop")
            .expect("drops Docker's bridges");
        let ssh = rules
            .iter()
            .position(|line| line.starts_with("tcp dport"))
            .expect("accepts SSH");
        assert!(containers < ssh);
        assert!(rules.contains(&"ip saddr 10.213.0.0/16 drop"));
    }

    #[test]
    fn containers_cannot_reach_the_tailnet_private_ranges_or_metadata() {
        let rules = rules();
        assert!(rules.contains(&"ip saddr 10.213.0.0/16 jump from_containers"));
        assert!(rules.contains(&"iifname \"br-*\" jump from_containers"));
        let at = |rule: &str| {
            rules
                .iter()
                .position(|line| *line == rule)
                .unwrap_or_else(|| panic!("{rule}"))
        };
        // The tailnet by its interface first: its subnet routes and exit nodes reach any
        // address.
        let chain = at("chain from_containers {");
        assert_eq!(rules[chain + 1], "oifname \"tailscale0\" drop");
        let own = at("ip daddr 10.213.0.0/16 return");
        let tailnet = at("ip daddr 100.64.0.0/10 drop");
        let dns = at("meta nfproto ipv4 meta l4proto { tcp, udp } th dport 53 return");
        let blocked = rules
            .iter()
            .position(|line| line.starts_with("ip daddr {"))
            .expect("blocked ranges");
        // The tailnet's DNS stays out of reach; the host's resolvers do not.
        assert!(own < tailnet && tailnet < dns && dns < blocked);
        for range in [
            "10.0.0.0/8",
            "172.16.0.0/12",
            "192.168.0.0/16",
            "169.254.0.0/16",
            "168.63.129.16",
        ] {
            assert!(rules[blocked].contains(range), "{range}");
        }
        assert!(rules.contains(&"ip6 daddr { fc00::/7, fe80::/10 } drop"));
        // The table replaces itself, in one transaction.
        assert_eq!(rules[..2], ["table inet brume", "delete table inet brume"]);
    }

    #[test]
    fn nothing_from_outside_the_bridges_opens_a_connection_to_a_container() {
        let rules = rules();
        let at = |rule: &str| {
            rules
                .iter()
                .position(|line| *line == rule)
                .unwrap_or_else(|| panic!("{rule}"))
        };
        let forward = at("chain forward {");
        // First, before Docker's own rules, which a flush of the ruleset deletes.
        assert_eq!(
            rules[forward + 2],
            "iifname != \"br-*\" oifname \"br-*\" ct state new drop"
        );
        assert!(forward + 2 < at("iifname \"br-*\" jump from_containers"));
    }

    #[test]
    fn the_table_survives_nftables_service() {
        let lines: Vec<&str> = DROP_IN_TEXT.lines().collect();
        let load = "/usr/sbin/nft -f /etc/brume/brume.nft";
        // Docker's limit of 3 starts a minute would leave it stopped after a few reloads.
        let reset = "-/usr/bin/systemctl reset-failed docker.service";
        // `--no-block`: Docker is ordered after nftables.service, so a blocking restart from
        // its own command would wait for itself.
        let restart = "-/usr/bin/systemctl --no-block try-restart docker.service";
        for key in ["ExecStartPost", "ExecReload", "ExecStopPost"] {
            assert_eq!(
                lines
                    .iter()
                    .filter_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                    .collect::<Vec<_>>(),
                [load, reset, restart],
                "{key}"
            );
        }
        // Appended to the unit's own commands, never reset.
        assert!(!lines.iter().any(|line| line.ends_with('=')));
        assert!(lines.contains(&"Before=docker.service"));
        assert_eq!(
            DROP_IN_PATH,
            "/etc/systemd/system/nftables.service.d/brume.conf"
        );
    }

    #[test]
    fn the_firewall_step_installs_the_drop_in() {
        let fake = FakeSystem::default();
        fake.answer(
            &systemd::is_enabled_query("brume-nftables.service"),
            Output::ok("enabled\n"),
        );
        fake.answer(
            &systemd::is_active_query("brume-nftables.service"),
            Output::ok("active\n"),
        );
        fake.answer(&super::list_table(), Output::ok("table inet brume {}\n"));
        let mut host = Host::new(Box::new(fake.clone()));
        assert!(!Firewall.is_done(&host).expect("reads"));
        Firewall.apply(&mut host).expect("applies");
        assert_eq!(fake.file(DROP_IN_PATH).as_deref(), Some(DROP_IN_TEXT));
        let changes = fake.changes();
        let at = |change: &str| {
            changes
                .iter()
                .position(|line| line == change)
                .unwrap_or_else(|| panic!("{change} in {changes:#?}"))
        };
        assert!(at(&format!("write {DROP_IN_PATH}")) < at("systemctl daemon-reload"));
        assert!(Firewall.is_done(&host).expect("reads"));
        // A changed drop-in is written again.
        fake.put_file(DROP_IN_PATH, "[Service]\n");
        assert!(!Firewall.is_done(&host).expect("reads"));
    }

    #[test]
    fn the_table_loads_before_docker() {
        assert!(UNIT_TEXT.contains("\nBefore=network-pre.target docker.service shutdown.target\n"));
        assert!(UNIT_TEXT.contains("\nWants=network-pre.target\n"));
        assert!(UNIT_TEXT.contains("\nRemainAfterExit=yes\n"));
        assert!(!UNIT_TEXT.contains("ExecStop"));
    }
}
