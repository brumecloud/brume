//! The Cloudflare checks and steps, against the fake API.

use std::rc::Rc;

use brume_domain::Zone;
use secrecy::SecretString;

use super::fake::{ACCOUNT, API_TOKEN, FakeCloudflare, State, ZONE_ID};
use super::{
    Client, Cloudflare, Permission, Site, check, hidden, no_terminal, parse_zone, steps,
    tunnel_name,
};
use crate::install::host::Host;
use crate::install::host::fake::FakeSystem;
use crate::install::wait::fake::FakeConsole;
use crate::install::{Outcome, run_steps};

const ZONE: &str = "example.com";
const MACHINE_ID: &str = "4c1f0e6b2a9d4e8f9b7a6c5d4e3f2a1b";

/// A host named `web-1`, whose engine runs.
fn host() -> FakeSystem {
    let fake = FakeSystem::default();
    fake.put_file("/proc/sys/kernel/hostname", "web-1\n");
    fake.put_file("/etc/machine-id", &format!("{MACHINE_ID}\n"));
    fake.start_engine();
    fake
}

/// This host's tunnel name.
fn our_tunnel(fake: &FakeSystem) -> String {
    tunnel_name(&Host::new(Box::new(fake.clone()))).expect("a name")
}

fn zone() -> Zone {
    parse_zone(ZONE).expect("a zone")
}

fn client(api: &FakeCloudflare) -> Client {
    Client::local(api.base(), SecretString::from(API_TOKEN))
}

/// Checks the zone, then runs the Cloudflare steps on `fake`.
fn install(
    api: &FakeCloudflare,
    fake: &FakeSystem,
) -> (anyhow::Result<Vec<Outcome>>, Site, String) {
    let client = client(api);
    let host = Host::new(Box::new(fake.clone()));
    let site = check(&host, &client, &zone()).expect("the checks pass");
    let cloudflare = Cloudflare {
        client: Rc::new(client),
        site: Rc::new(site.clone()),
    };
    let console = FakeConsole::new();
    let all = steps(&cloudflare, &console.console);
    let mut host = Host::new(Box::new(fake.clone()));
    let mut out = Vec::new();
    let outcomes = run_steps(&all, &mut host, &mut out);
    let printed = format!(
        "{}{}",
        String::from_utf8(out).expect("UTF-8"),
        console.printed()
    );
    (outcomes, site, printed)
}

#[test]
fn tunnel_name_is_stable_and_per_machine() {
    let fake = host();
    let name = our_tunnel(&fake);
    assert!(name.starts_with("brume-web-1-"), "{name}");
    assert_eq!(name.len(), "brume-web-1-".len() + 8);
    assert_eq!(our_tunnel(&fake), name);
    // Another server of the same name gets another tunnel.
    let other = host();
    other.put_file("/etc/machine-id", "0f0e0d0c0b0a09080706050403020100\n");
    assert_ne!(our_tunnel(&other), name);
    // A name of any shape becomes a label.
    fake.put_file("/proc/sys/kernel/hostname", "Web_1.Example.org\n");
    assert!(our_tunnel(&fake).starts_with("brume-web-1-"));
    fake.put_file("/proc/sys/kernel/hostname", "---\n");
    assert!(our_tunnel(&fake).starts_with("brume-host-"));
    fake.put_file("/etc/machine-id", "\n");
    assert!(tunnel_name(&Host::new(Box::new(fake))).is_err());
}

#[test]
fn the_zone_is_checked_before_cloudflare() {
    assert_eq!(zone().as_str(), "example.com");
    for invalid in [
        "Example.com",
        "com",
        "example..com",
        "-x.com",
        "example.com.",
    ] {
        let error = parse_zone(invalid).expect_err(invalid);
        assert!(error.to_string().contains("--domain"), "{error}");
    }
}

#[test]
fn a_fresh_zone_gets_the_tunnel_the_wildcard_and_https() {
    let api = FakeCloudflare::start(State::new(ZONE));
    let fake = host();
    let (outcomes, site, printed) = install(&api, &fake);
    assert_eq!(outcomes.expect("installs"), vec![Outcome::Applied; 4]);
    assert_eq!(site.tunnel_name, our_tunnel(&fake));

    let state = api.state();
    let [tunnel] = state.tunnels.as_slice() else {
        panic!("one tunnel: {:?}", state.tunnels);
    };
    assert_eq!(tunnel.name, site.tunnel_name);
    let [record] = state.records.as_slice() else {
        panic!("one record: {:?}", state.records);
    };
    assert_eq!(
        (record.kind.as_str(), record.name.as_str(), record.proxied),
        ("CNAME", "*.example.com", true)
    );
    assert_eq!(record.content, format!("{}.cfargotunnel.com", tunnel.id));
    assert_eq!(state.always_use_https, "on");

    // The engine has the zone, the tunnel and its token.
    let (ingress, token) = fake.engine_ingress().expect("configured");
    assert_eq!(ingress.zone, ZONE);
    assert_eq!(ingress.tunnel_id, tunnel.id);
    assert_eq!(token, tunnel.token);
    assert_eq!(
        fake.changes(),
        [format!("admin ConfigureIngress example.com {}", tunnel.id)]
    );
    assert!(printed.contains(&format!(
        "- Create the Cloudflare tunnel {}… done",
        site.tunnel_name
    )));
    drop(state);

    // Run again: everything is done, nothing is written.
    api.state().requests.clear();
    fake.clear_changes();
    let (outcomes, _, printed) = install(&api, &fake);
    assert_eq!(outcomes.expect("runs"), vec![Outcome::AlreadyDone; 4]);
    assert_eq!(fake.changes(), Vec::<String>::new());
    // The checks probe the permissions to edit DNS and settings with writes that change
    // nothing, and the tunnels' with a read; the steps write nothing.
    let state = api.state();
    assert_eq!(state.tunnels.len(), 1);
    assert_eq!(state.records.len(), 1);
    assert_eq!(
        state.writes(),
        [
            format!("POST /zones/{ZONE_ID}/dns_records/batch"),
            format!("PATCH /zones/{ZONE_ID}/settings/always_use_https"),
        ]
    );
    assert_eq!(printed.matches("already done").count(), 4, "{printed}");
}

#[test]
fn reuses_existing_tunnel() {
    let fake = host();
    let mut state = State::new(ZONE);
    // A deleted tunnel of the same name, and the live one of a previous install.
    let deleted = state.add_tunnel(&our_tunnel(&fake));
    state.tunnels[0].deleted = true;
    let live = state.add_tunnel(&our_tunnel(&fake));
    // Its wildcard is there, not proxied yet.
    state.add_record(
        "CNAME",
        "*.example.com",
        &format!("{live}.cfargotunnel.com"),
        false,
    );
    let api = FakeCloudflare::start(state);
    let (outcomes, _, _) = install(&api, &fake);
    assert_eq!(
        outcomes.expect("installs"),
        [
            Outcome::AlreadyDone,
            Outcome::Applied,
            Outcome::Applied,
            Outcome::Applied
        ]
    );
    let state = api.state();
    // No tunnel created, nor asked to be.
    assert_eq!(state.tunnels.len(), 2);
    let created = state
        .requests
        .iter()
        .filter(|request| request.starts_with("POST /accounts/"))
        .count();
    assert_eq!(created, 0);
    // The record is kept, and proxied.
    assert_eq!(state.records.len(), 1);
    assert!(state.records[0].proxied);
    let (ingress, token) = fake.engine_ingress().expect("configured");
    assert_eq!(ingress.tunnel_id, live);
    assert_ne!(ingress.tunnel_id, deleted);
    assert_eq!(token, state.tunnels[1].token);
}

#[test]
fn cloudflare_refuses_foreign_wildcard() {
    let fake = host();
    for (kind, content, shown) in [
        ("A", "203.0.113.7", "A 203.0.113.7"),
        (
            "CNAME",
            "0b1c2d3e-0000-4000-8000-000000000000.cfargotunnel.com",
            "CNAME 0b1c2d3e-0000-4000-8000-000000000000.cfargotunnel.com",
        ),
    ] {
        let mut state = State::new(ZONE);
        state.add_record(kind, "*.example.com", content, true);
        let api = FakeCloudflare::start(state);
        let host = Host::new(Box::new(fake.clone()));
        let error = check(&host, &client(&api), &zone()).expect_err("refused");
        let message = error.to_string();
        assert!(
            message.starts_with(&format!("*.example.com already has a DNS record: {shown}.")),
            "{message}"
        );
        assert!(message.ends_with("Nothing was changed."), "{message}");
        let state = api.state();
        assert!(state.tunnels.is_empty());
        assert_eq!(state.records.len(), 1);
        assert_eq!(state.always_use_https, "off");
    }
}

#[test]
fn cloudflare_names_missing_permission() {
    let fake = host();
    let host = Host::new(Box::new(fake.clone()));
    for (denied, lacks) in [
        (
            vec![Permission::DnsEdit, Permission::TunnelEdit],
            "lacks Zone → DNS → Edit, Account → Cloudflare Tunnel → Edit (or Cloudflare One \
             Connector: cloudflared → Write) on example.com",
        ),
        (
            vec![Permission::ZoneSettingsEdit],
            "lacks Zone → Zone Settings → Edit on example.com",
        ),
        (vec![Permission::ZoneRead], "no zone example.com"),
    ] {
        let mut state = State::new(ZONE);
        state.denied = denied.iter().copied().collect();
        let api = FakeCloudflare::start(state);
        let error = check(&host, &client(&api), &zone()).expect_err("refused");
        let message = error.to_string();
        assert!(message.contains(lacks), "{message}");
        // The admin learns all that the token needs.
        assert!(
            message.contains(
                "Zone → Zone → Read, Zone → DNS → Edit, Zone → Zone Settings → Edit, Account → \
                 Cloudflare Tunnel → Edit (or Cloudflare One Connector: cloudflared → Write)"
            ),
            "{message}"
        );
        let state = api.state();
        assert!(state.tunnels.is_empty() && state.records.is_empty());
        assert_eq!(state.always_use_https, "off");
    }
}

#[test]
fn token_never_in_debug_output() {
    let api = FakeCloudflare::start(State::new(ZONE));
    let fake = host();
    let client = client(&api);
    let debug = format!("{client:?}");
    assert!(!debug.contains(API_TOKEN), "{debug}");

    // A server that echoes the token in its error: the error does not.
    let wrong = "cf-wrong-token-7Hq2";
    let host = Host::new(Box::new(fake.clone()));
    let error = check(
        &host,
        &Client::local(api.base(), SecretString::from(wrong)),
        &zone(),
    )
    .expect_err("an invalid token");
    for shown in [
        error.to_string(),
        format!("{error:#}"),
        format!("{error:?}"),
    ] {
        assert!(!shown.contains(wrong), "{shown}");
        assert!(
            shown.contains("Invalid access token: Bearer <redacted>"),
            "{shown}"
        );
    }
    assert!(error.to_string().contains("Check the API token"), "{error}");

    // A whole install: neither token is in what it prints, records or keeps.
    let (outcomes, site, printed) = install(&api, &fake);
    outcomes.expect("installs");
    let tunnel_token = api.state().tunnels[0].token.clone();
    let (_, given) = fake.engine_ingress().expect("configured");
    assert_eq!(given, tunnel_token);
    let cloudflare = Cloudflare {
        client: Rc::new(client),
        site: Rc::new(site),
    };
    for text in [
        printed,
        fake.changes().join("\n"),
        format!("{cloudflare:?}"),
    ] {
        assert!(!text.contains(API_TOKEN), "{text}");
        assert!(!text.contains(&tunnel_token), "{text}");
    }
}

#[test]
fn an_engine_refusal_gives_its_reason() {
    let api = FakeCloudflare::start(State::new(ZONE));
    let fake = host();
    fake.refuse_admin("Another server writes to this bucket.");
    let (outcomes, _, _) = install(&api, &fake);
    let error = outcomes.expect_err("refused");
    assert!(
        format!("{error:#}").contains("Another server writes to this bucket."),
        "{error:#}"
    );
}

#[test]
fn a_pending_zone_is_noted() {
    let mut state = State::new(ZONE);
    "pending".clone_into(&mut state.zone_status);
    let api = FakeCloudflare::start(state);
    let host = Host::new(Box::new(host()));
    let site = check(&host, &client(&api), &zone()).expect("passes");
    assert!(!site.zone_active());
    assert_eq!(site.zone_status, "pending");
}

/// The probe of the right to edit tunnels: a read of a tunnel that does not exist.
fn tunnel_probe() -> String {
    format!("GET /accounts/{ACCOUNT}/cfd_tunnel/00000000-0000-4000-8000-000000000000/token")
}

#[test]
fn the_tunnel_check_reads_and_creates_nothing() {
    let fake = host();
    let host = Host::new(Box::new(fake.clone()));
    // A token that may list the tunnels, and not create one.
    let mut state = State::new(ZONE);
    state.denied.insert(Permission::TunnelEdit);
    state.tunnel_read = true;
    let api = FakeCloudflare::start(state);
    let error = check(&host, &client(&api), &zone()).expect_err("refused");
    assert!(
        error
            .to_string()
            .contains("lacks Account → Cloudflare Tunnel → Edit (or Cloudflare One Connector"),
        "{error}"
    );
    let state = api.state();
    assert!(state.tunnels.is_empty());
    assert!(
        state.requests.contains(&tunnel_probe()),
        "{:#?}",
        state.requests
    );
    // No call to the tunnels but reads: none can create one.
    let tunnel_writes: Vec<_> = state
        .writes()
        .into_iter()
        .filter(|request| request.contains("/cfd_tunnel"))
        .collect();
    assert_eq!(tunnel_writes, Vec::<String>::new());
    drop(state);

    // A token that may: the same read passes.
    let api = FakeCloudflare::start(State::new(ZONE));
    check(&host, &client(&api), &zone()).expect("passes");
    let state = api.state();
    assert!(
        state.requests.contains(&tunnel_probe()),
        "{:#?}",
        state.requests
    );
    assert!(
        state
            .writes()
            .iter()
            .all(|request| !request.contains("/cfd_tunnel")),
        "{:#?}",
        state.writes()
    );
}

#[test]
fn an_unexpected_answer_to_a_permission_check_is_an_error() {
    let fake = host();
    let host = Host::new(Box::new(fake.clone()));
    for (call, status, code) in [
        (tunnel_probe(), 429, 971),
        (
            format!("PATCH /zones/{ZONE_ID}/settings/always_use_https"),
            500,
            1000,
        ),
        (
            format!("POST /zones/{ZONE_ID}/dns_records/batch"),
            409,
            1000,
        ),
    ] {
        let mut state = State::new(ZONE);
        state.forced.push((call.clone(), status, code));
        let api = FakeCloudflare::start(state);
        let error = check(&host, &client(&api), &zone()).expect_err(&call);
        let message = error.to_string();
        assert!(
            message.contains(&format!("with HTTP {status}")),
            "{call}: {message}"
        );
        assert!(message.contains("did not expect"), "{call}: {message}");
        assert!(message.contains("Nothing was changed"), "{call}: {message}");
        assert!(!message.contains("lacks"), "{call}: {message}");
    }
}

#[test]
fn a_renamed_server_finds_its_tunnel() {
    let fake = host();
    // The first install ran while the server was named `old-name`.
    fake.put_file("/proc/sys/kernel/hostname", "old-name\n");
    let old = our_tunnel(&fake);
    fake.put_file("/proc/sys/kernel/hostname", "web-1\n");
    assert_ne!(our_tunnel(&fake), old);
    let mut state = State::new(ZONE);
    // Another server's tunnel, and this server's, with the wildcard pointing to it.
    state.add_tunnel("brume-web-1-0a0b0c0d");
    let ours = state.add_tunnel(&old);
    state.add_record(
        "CNAME",
        "*.example.com",
        &format!("{ours}.cfargotunnel.com"),
        true,
    );
    "on".clone_into(&mut state.always_use_https);
    let api = FakeCloudflare::start(state);

    let (outcomes, site, _) = install(&api, &fake);
    assert_eq!(site.tunnel_name, old);
    assert_eq!(
        outcomes.expect("installs"),
        [
            Outcome::AlreadyDone,
            Outcome::AlreadyDone,
            Outcome::AlreadyDone,
            Outcome::Applied
        ]
    );
    let (ingress, _) = fake.engine_ingress().expect("configured");
    assert_eq!(ingress.tunnel_id, ours);
    let state = api.state();
    assert_eq!(state.tunnels.len(), 2);
    assert!(
        state
            .requests
            .iter()
            .any(|request| request.contains("cfd_tunnel?name_prefix=brume-")),
        "{:#?}",
        state.requests
    );
}

#[test]
fn the_token_stays_out_of_ps_and_history_without_a_terminal() {
    let message = no_terminal().to_string();
    assert!(
        message.contains("read -rs CLOUDFLARE_API_TOKEN; export CLOUDFLARE_API_TOKEN"),
        "{message}"
    );
    assert!(
        message.contains("sudo --preserve-env=CLOUDFLARE_API_TOKEN"),
        "{message}"
    );
    // Never on a command line, where `ps` and sudo's log show it.
    assert!(!message.contains("CLOUDFLARE_API_TOKEN="), "{message}");
}

#[test]
fn the_echo_comes_back_on_every_path() {
    use std::cell::RefCell;

    let settings = RefCell::new(Vec::new());
    let stty = |setting: &str| {
        settings.borrow_mut().push(setting.to_owned());
        Ok(true)
    };
    // Read, failed, panicked: the echo is on again.
    let read = hidden(&stty, || Ok("token".to_owned())).expect("reads");
    assert_eq!(read, "token");
    hidden::<String>(&stty, || anyhow::bail!("cannot read the token")).expect_err("fails");
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        hidden::<String>(&stty, || panic!("interrupted"))
    }));
    assert!(panicked.is_err());
    assert_eq!(
        *settings.borrow(),
        ["-echo", "echo", "-echo", "echo", "-echo", "echo"]
    );

    // `stty -echo` fails: no read, and the echo is set on anyway.
    settings.borrow_mut().clear();
    let refuse = |setting: &str| {
        settings.borrow_mut().push(setting.to_owned());
        Ok(setting == "echo")
    };
    let error = hidden(&refuse, || Ok("token".to_owned())).expect_err("no hidden input");
    assert!(error.to_string().contains("stty -echo"), "{error}");
    assert_eq!(*settings.borrow(), ["-echo", "echo"]);
}
