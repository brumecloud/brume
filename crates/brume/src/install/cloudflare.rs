//! Cloudflare: the public entry of the apps. A tunnel in local config carries their traffic,
//! the proxied wildcard `*.<zone>` points to it, and the zone redirects HTTP to HTTPS. The
//! engine gets the tunnel's token, sealed by its master key, and the agent runs cloudflared.
//!
//! Before anything changes on the host, `check` makes sure that the token can do all of it,
//! and that no other record holds the wildcard: the install refuses up front, naming the
//! missing permission or the record. The API token stays in memory: read from
//! `CLOUDFLARE_API_TOKEN` or typed on the terminal, never written to a file, never passed to a
//! command.

mod api;
#[cfg(test)]
pub mod fake;

use std::collections::BTreeSet;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::Duration;

use anyhow::{Context as _, bail};
use brume_domain::Zone;
use brume_proto::admin::{AdminReply, AdminRequest};
use reqwest::Method;
use secrecy::SecretString;
use serde_json::json;
use sha2::{Digest as _, Sha256};

pub use self::api::Client;
use self::api::{ApiError, DnsRecord, TunnelInfo};
use super::Step;
use super::host::Host;
use super::wait::Console;
use crate::admin::is_refusal;

/// The variable that holds the API token. `sudo` drops it unless told to keep it.
pub const TOKEN_ENV: &str = "CLOUDFLARE_API_TOKEN";

/// The longest API token accepted. Cloudflare's are 40 characters.
const MAX_TOKEN_BYTES: usize = 1024;

/// The zone setting that redirects HTTP to HTTPS.
const ALWAYS_USE_HTTPS: &str = "always_use_https";

/// A tunnel's DNS name is `<id>.cfargotunnel.com`.
const TUNNEL_DOMAIN: &str = "cfargotunnel.com";

/// The longest wait for the engine's `admin.sock`, after the services start.
const ENGINE_TIMEOUT: Duration = Duration::from_secs(60);

/// How often the engine is asked while it starts.
const ENGINE_POLL: Duration = Duration::from_millis(500);

/// The longest host name kept in a tunnel's name.
const MAX_HOST_LABEL: usize = 40;

/// The start of the names of Brume's tunnels.
const TUNNEL_PREFIX: &str = "brume-";

/// A well-formed tunnel id that no tunnel has. Reading its token needs the right to edit the
/// account's tunnels, and finds nothing.
const PROBE_TUNNEL_ID: &str = "00000000-0000-4000-8000-000000000000";

/// A permission of a Cloudflare API token, as the dashboard names it. The dashboard shows the
/// tunnels' under either of two names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    ZoneRead,
    DnsEdit,
    ZoneSettingsEdit,
    TunnelEdit,
}

impl Permission {
    const ALL: [Self; 4] = [
        Self::ZoneRead,
        Self::DnsEdit,
        Self::ZoneSettingsEdit,
        Self::TunnelEdit,
    ];
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZoneRead => "Zone → Zone → Read",
            Self::DnsEdit => "Zone → DNS → Edit",
            Self::ZoneSettingsEdit => "Zone → Zone Settings → Edit",
            Self::TunnelEdit => {
                "Account → Cloudflare Tunnel → Edit (or Cloudflare One Connector: cloudflared → \
                 Write)"
            }
        })
    }
}

/// The permissions of the token, as a list for the admin.
fn permission_list() -> String {
    Permission::ALL
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The zone and the tunnel of the install, as `check` found them.
#[derive(Clone, Debug)]
pub struct Site {
    pub zone: Zone,
    zone_id: String,
    /// `active`, or `pending` until the registrar points to Cloudflare.
    pub zone_status: String,
    account_id: String,
    /// The name of this server's tunnel: the one found, else the one to create.
    pub tunnel_name: String,
    /// The end of the names of this server's tunnels: `-` and a hash of its machine id.
    tunnel_suffix: String,
}

impl Site {
    pub fn zone_active(&self) -> bool {
        self.zone_status == "active"
    }

    fn wildcard(&self) -> String {
        format!("*.{}", self.zone)
    }
}

/// What the Cloudflare steps share: the client, with the token, and the site.
#[derive(Clone, Debug)]
pub struct Cloudflare {
    pub client: Rc<Client>,
    pub site: Rc<Site>,
}

/// `--domain`, checked as the engine checks it.
pub fn parse_zone(domain: &str) -> anyhow::Result<Zone> {
    Zone::parse(domain).map_err(|error| {
        anyhow::anyhow!(
            "--domain {domain:?} is invalid: the zone {}.",
            error.message
        )
    })
}

/// The API token: `CLOUDFLARE_API_TOKEN`, or typed on the terminal without echo.
pub fn read_token(zone: &Zone) -> anyhow::Result<SecretString> {
    match std::env::var(TOKEN_ENV) {
        Ok(raw) if !raw.trim().is_empty() => token_from(&raw),
        _ => prompt_token(zone),
    }
}

/// A token, trimmed, if it can be one. The error never quotes it.
fn token_from(raw: &str) -> anyhow::Result<SecretString> {
    let token = raw.trim();
    if token.is_empty()
        || token.len() > MAX_TOKEN_BYTES
        || !token.bytes().all(|b| b.is_ascii_graphic())
    {
        bail!(
            "The Cloudflare API token is not one: it must be 1 to {MAX_TOKEN_BYTES} printable \
             characters, without spaces."
        );
    }
    Ok(SecretString::from(token))
}

/// The error when there is no terminal to type the token on. The token goes through the
/// environment: on a command line, `ps` and sudo's log would show it.
fn no_terminal() -> anyhow::Error {
    anyhow::anyhow!(
        "--domain needs a Cloudflare API token, and there is no terminal to type it. Give it in \
         {TOKEN_ENV}, kept through sudo: `read -rs {TOKEN_ENV}; export {TOKEN_ENV}` (it stays \
         out of the shell's history), then `sudo --preserve-env={TOKEN_ENV} brume install …`, \
         or `curl … | sudo --preserve-env={TOKEN_ENV} bash -s -- …`."
    )
}

/// Asks for the token on the terminal, without echo. `stty` turns the echo off and on: no
/// unsafe code, and no crate.
fn prompt_token(zone: &Zone) -> anyhow::Result<SecretString> {
    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map_err(|_| no_terminal())?;
    writeln!(
        tty,
        "The public domains of {zone} need a Cloudflare API token with these permissions: \
         {list}, on the zone {zone} and its account. Create one in Cloudflare's dashboard: My \
         Profile → API Tokens, or Manage Account → Account API Tokens. It is used during the \
         install, then forgotten. ({TOKEN_ENV} gives it without this question.)",
        list = permission_list(),
    )?;
    write!(tty, "Cloudflare API token (hidden): ")?;
    tty.flush()?;
    let line = hidden(&|setting| stty(&tty, setting), || {
        let mut line = String::new();
        BufReader::new((&tty).take(MAX_TOKEN_BYTES as u64 + 2))
            .read_line(&mut line)
            .context("cannot read the token")?;
        Ok(line)
    })?;
    writeln!(tty)?;
    token_from(&line)
}

/// Sets the terminal `tty`, such as `-echo`, and returns whether `stty` could.
fn stty(tty: &File, setting: &str) -> anyhow::Result<bool> {
    let status = Command::new("stty")
        .arg(setting)
        .env_remove(TOKEN_ENV)
        .stdin(Stdio::from(tty.try_clone()?))
        .status()
        .context("cannot run stty")?;
    Ok(status.success())
}

/// A terminal setting, such as `-echo`, made by `stty`: whether it could.
type Stty<'a> = &'a dyn Fn(&str) -> anyhow::Result<bool>;

/// Runs `read` with the terminal's echo off, and turns the echo back on however `read` ends:
/// it returns, fails or panics.
///
/// Ctrl-C kills the install before the echo comes back: the shell then restores the terminal
/// it had, as bash and zsh do after a command that a signal killed, and `stty echo` restores
/// it elsewhere. Restoring it here would need a signal handler: unsafe code, or a crate.
fn hidden<T>(stty: Stty<'_>, read: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<T> {
    let _echo = Echo::off(stty)?;
    read()
}

/// The terminal's echo, off until dropped.
struct Echo<'a> {
    stty: Stty<'a>,
}

impl<'a> Echo<'a> {
    fn off(stty: Stty<'a>) -> anyhow::Result<Self> {
        // Made first: a `stty -echo` that fails may have turned the echo off.
        let echo = Self { stty };
        if !(echo.stty)("-echo")? {
            bail!("Cannot hide the token as it is typed: `stty -echo` failed.");
        }
        Ok(echo)
    }
}

impl Drop for Echo<'_> {
    fn drop(&mut self) {
        let _ = (self.stty)("echo");
    }
}

/// The tunnel's name: `brume-`, the host name, and a hash of the machine id.
#[cfg(test)]
pub fn tunnel_name(host: &Host) -> anyhow::Result<String> {
    Ok(tunnel_names(host)?.0)
}

/// The name of a new tunnel for this server, and the end of the names of its tunnels: `-` and
/// a hash of the machine id. The hash never changes: a rerun finds the server's tunnel by it,
/// after a change of host name too, and never another server's of the same host name.
fn tunnel_names(host: &Host) -> anyhow::Result<(String, String)> {
    let hostname = host
        .read(Path::new("/proc/sys/kernel/hostname"))?
        .unwrap_or_default();
    let machine_id = host.read(Path::new("/etc/machine-id"))?.unwrap_or_default();
    let machine_id = machine_id.trim();
    if machine_id.is_empty() {
        bail!("/etc/machine-id is missing or empty: cannot name this server's tunnel.");
    }
    let mut label: String = hostname
        .trim()
        .split('.')
        .next()
        .unwrap_or_default()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .take(MAX_HOST_LABEL)
        .collect();
    label = label.trim_matches('-').to_owned();
    if label.is_empty() {
        "host".clone_into(&mut label);
    }
    let digest = Sha256::digest(format!("brume tunnel {machine_id}"));
    let suffix = format!("-{}", hex::encode(&digest[..4]));
    Ok((format!("{TUNNEL_PREFIX}{label}{suffix}"), suffix))
}

/// The DNS name of the tunnel `id`.
fn tunnel_target(id: &str) -> String {
    format!("{id}.{TUNNEL_DOMAIN}")
}

/// Whether `record` points to the tunnel `id`.
fn points_to(record: &DnsRecord, id: &str) -> bool {
    record.kind == "CNAME"
        && record
            .content
            .trim_end_matches('.')
            .eq_ignore_ascii_case(&tunnel_target(id))
}

/// The records that hold `*.<zone>` for something else than the tunnel, as the admin reads
/// them: `A 203.0.113.7`.
fn foreign(records: &[DnsRecord], tunnel: Option<&TunnelInfo>) -> Vec<String> {
    records
        .iter()
        .filter(|record| !tunnel.is_some_and(|tunnel| points_to(record, &tunnel.id)))
        .map(|record| format!("{} {}", record.kind, record.content))
        .collect()
}

/// The records named `*.<zone>`.
fn wildcard_records(client: &Client, site: &Site) -> Result<Vec<DnsRecord>, ApiError> {
    let name = site.wildcard();
    let records = client.records(&site.zone_id, &name)?;
    Ok(records
        .into_iter()
        .filter(|record| record.name.eq_ignore_ascii_case(&name))
        .collect())
}

/// This server's live tunnel: one named `brume-<host name><suffix>`, under the current name
/// first.
fn find_tunnel(client: &Client, site: &Site) -> Result<Option<TunnelInfo>, ApiError> {
    let suffix = &site.tunnel_suffix;
    let mut ours: Vec<TunnelInfo> = client
        .tunnels(&site.account_id, TUNNEL_PREFIX)?
        .into_iter()
        .filter(|tunnel| {
            tunnel.deleted_at.is_none()
                && tunnel.name.len() > TUNNEL_PREFIX.len() + suffix.len()
                && tunnel.name.starts_with(TUNNEL_PREFIX)
                && tunnel.name.ends_with(suffix.as_str())
        })
        .collect();
    if ours.is_empty() {
        return Ok(None);
    }
    let current = ours
        .iter()
        .position(|tunnel| tunnel.name == site.tunnel_name)
        .unwrap_or(0);
    Ok(Some(ours.swap_remove(current)))
}

/// Records `permission` as missing if `result` says so, and returns the rest.
fn note<T>(
    missing: &mut BTreeSet<Permission>,
    permission: Permission,
    result: Result<T, ApiError>,
) -> anyhow::Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.denied() => {
            missing.insert(permission);
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

/// Whether the token has `permission`, as the call `method path` tells: Cloudflare refuses it
/// for a missing permission, or answers one of the `expected` statuses. Any other answer, such
/// as 429 when Cloudflare throttles the token, is an error that says so.
fn may(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<&serde_json::Value>,
    expected: &[u16],
    permission: Permission,
) -> anyhow::Result<bool> {
    client.may(method, path, body, expected).map_err(|error| {
        if error.answered() {
            anyhow::anyhow!(
                "{error}. Brume did not expect this answer as it checked that the token has \
                 {permission}. Nothing was changed: run the same command again in a minute."
            )
        } else {
            error.into()
        }
    })
}

/// Checks, before any change, that the token can set up `zone`, and that no other record
/// holds `*.<zone>`. Creates and changes nothing: the right to edit tunnels is probed with a
/// read that needs it, the others with writes that change nothing.
pub fn check(host: &Host, client: &Client, zone: &Zone) -> anyhow::Result<Site> {
    let (tunnel_name, tunnel_suffix) = tunnel_names(host)?;
    let zones = client.zones(zone.as_str()).map_err(|error| {
        let hint = if error.answered() {
            " Check the API token: it may be mistyped, expired or revoked."
        } else {
            ""
        };
        anyhow::anyhow!("{error}.{hint}")
    })?;
    let mut found: Vec<_> = zones
        .into_iter()
        .filter(|found| found.name == zone.as_str())
        .collect();
    if found.len() > 1 {
        found.retain(|found| found.status == "active");
    }
    let info = match found.as_slice() {
        [info] => info.clone(),
        [] => bail!(
            "Cloudflare shows the token no zone {zone}. Add {zone} to your Cloudflare account, \
             and give the token {} on it. It needs: {}.",
            Permission::ZoneRead,
            permission_list()
        ),
        _ => bail!(
            "The token reaches {zone} in several Cloudflare accounts: limit it to the account \
             that serves {zone}."
        ),
    };
    let mut site = Site {
        zone: zone.clone(),
        zone_id: info.id,
        zone_status: info.status,
        account_id: info.account.id,
        tunnel_name,
        tunnel_suffix,
    };

    let mut missing = BTreeSet::new();
    let tunnel = note(
        &mut missing,
        Permission::TunnelEdit,
        find_tunnel(client, &site),
    )?;
    // The token of a tunnel that does not exist: Cloudflare answers 404 if the token may edit
    // tunnels, and refuses the call if not.
    let probe = format!(
        "/accounts/{}/cfd_tunnel/{PROBE_TUNNEL_ID}/token",
        site.account_id
    );
    if tunnel.is_some()
        && !may(
            client,
            Method::GET,
            &probe,
            None,
            &[404],
            Permission::TunnelEdit,
        )?
    {
        missing.insert(Permission::TunnelEdit);
    }
    let tunnel = tunnel.flatten();
    if let Some(tunnel) = &tunnel {
        tunnel.name.clone_into(&mut site.tunnel_name);
    }

    let records = note(
        &mut missing,
        Permission::DnsEdit,
        wildcard_records(client, &site),
    )?;
    // An empty batch of DNS changes changes nothing, whether Cloudflare takes it or refuses it
    // as invalid.
    let batch = format!("/zones/{}/dns_records/batch", site.zone_id);
    if records.is_some()
        && !may(
            client,
            Method::POST,
            &batch,
            Some(&json!({})),
            &[200, 400],
            Permission::DnsEdit,
        )?
    {
        missing.insert(Permission::DnsEdit);
    }

    let setting = note(
        &mut missing,
        Permission::ZoneSettingsEdit,
        client.setting(&site.zone_id, ALWAYS_USE_HTTPS),
    )?;
    // Sets the setting to what it is.
    if let Some(value) = setting {
        let path = format!("/zones/{}/settings/{ALWAYS_USE_HTTPS}", site.zone_id);
        let body = json!({ "value": value });
        if !may(
            client,
            Method::PATCH,
            &path,
            Some(&body),
            &[200],
            Permission::ZoneSettingsEdit,
        )? {
            missing.insert(Permission::ZoneSettingsEdit);
        }
    }

    if !missing.is_empty() {
        let lacks: Vec<String> = missing.iter().map(ToString::to_string).collect();
        bail!(
            "The Cloudflare API token lacks {} on {zone}. Add {} to the token in Cloudflare's \
             dashboard, then run the same command again. It needs: {}. Nothing was changed.",
            lacks.join(", "),
            if missing.len() == 1 { "it" } else { "them" },
            permission_list()
        );
    }
    let foreign = foreign(records.as_deref().unwrap_or_default(), tunnel.as_ref());
    if !foreign.is_empty() {
        bail!(
            "{wildcard} already has a DNS record: {}. Brume points {wildcard} to its tunnel, \
             and does not replace a record it did not create. Delete it in Cloudflare's DNS, or \
             install with another --domain, then run the same command again. Nothing was \
             changed.",
            foreign.join(", "),
            wildcard = site.wildcard()
        );
    }
    Ok(site)
}

/// The Cloudflare steps, in order: they run after the services start, the last one hands the
/// tunnel to the engine.
pub fn steps(cloudflare: &Cloudflare, console: &Rc<Console>) -> Vec<Box<dyn Step>> {
    let site = &cloudflare.site;
    vec![
        Box::new(Tunnel {
            cloudflare: cloudflare.clone(),
            name: format!("Create the Cloudflare tunnel {}", site.tunnel_name),
        }),
        Box::new(Wildcard {
            cloudflare: cloudflare.clone(),
            name: format!("Point {} to the tunnel (proxied CNAME)", site.wildcard()),
        }),
        Box::new(AlwaysHttps {
            cloudflare: cloudflare.clone(),
            name: format!("Turn on Always Use HTTPS on {}", site.zone),
        }),
        Box::new(Ingress {
            cloudflare: cloudflare.clone(),
            console: Rc::clone(console),
        }),
    ]
}

/// The error of a write that Cloudflare refused, naming the permission it needs if that is
/// why.
fn refused(error: ApiError, permission: Permission) -> anyhow::Error {
    if error.denied() {
        anyhow::anyhow!("{error}. The token needs {permission}.")
    } else {
        error.into()
    }
}

/// Creates the tunnel, in local config, unless this server's exists.
struct Tunnel {
    cloudflare: Cloudflare,
    name: String,
}

impl Step for Tunnel {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_done(&self, _host: &Host) -> anyhow::Result<bool> {
        Ok(find_tunnel(&self.cloudflare.client, &self.cloudflare.site)?.is_some())
    }

    fn apply(&self, _host: &mut Host) -> anyhow::Result<()> {
        let site = &self.cloudflare.site;
        self.cloudflare
            .client
            .create_tunnel(&site.account_id, &site.tunnel_name)
            .map_err(|error| refused(error, Permission::TunnelEdit))?;
        Ok(())
    }
}

/// The tunnel, which the step before created.
fn tunnel(cloudflare: &Cloudflare) -> anyhow::Result<TunnelInfo> {
    find_tunnel(&cloudflare.client, &cloudflare.site)?.with_context(|| {
        format!(
            "The Cloudflare tunnel {} is gone. Run the same command again: it creates it.",
            cloudflare.site.tunnel_name
        )
    })
}

/// Points `*.<zone>` to the tunnel, proxied: each subdomain of the zone reaches Traefik.
struct Wildcard {
    cloudflare: Cloudflare,
    name: String,
}

impl Step for Wildcard {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_done(&self, _host: &Host) -> anyhow::Result<bool> {
        let Cloudflare { client, site } = &self.cloudflare;
        let Some(tunnel) = find_tunnel(client, site)? else {
            return Ok(false);
        };
        let records = wildcard_records(client, site)?;
        Ok(records
            .iter()
            .any(|record| points_to(record, &tunnel.id) && record.proxied))
    }

    fn apply(&self, _host: &mut Host) -> anyhow::Result<()> {
        let Cloudflare { client, site } = &self.cloudflare;
        let tunnel = tunnel(&self.cloudflare)?;
        let records = wildcard_records(client, site)?;
        let foreign = foreign(&records, Some(&tunnel));
        if !foreign.is_empty() {
            bail!(
                "{} now has a DNS record: {}. Delete it in Cloudflare's DNS, then run the same \
                 command again.",
                site.wildcard(),
                foreign.join(", ")
            );
        }
        let result = match records.iter().find(|record| points_to(record, &tunnel.id)) {
            Some(record) => client.proxy_record(&site.zone_id, &record.id),
            None => {
                client.create_cname(&site.zone_id, &site.wildcard(), &tunnel_target(&tunnel.id))
            }
        };
        result.map_err(|error| refused(error, Permission::DnsEdit))
    }
}

/// Turns on "Always Use HTTPS": HTTP requests get a redirect. No HSTS: it is a setting of the
/// whole zone, which may serve other sites.
struct AlwaysHttps {
    cloudflare: Cloudflare,
    name: String,
}

impl Step for AlwaysHttps {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_done(&self, _host: &Host) -> anyhow::Result<bool> {
        let Cloudflare { client, site } = &self.cloudflare;
        Ok(client.setting(&site.zone_id, ALWAYS_USE_HTTPS)? == "on")
    }

    fn apply(&self, _host: &mut Host) -> anyhow::Result<()> {
        let Cloudflare { client, site } = &self.cloudflare;
        client
            .set_setting(&site.zone_id, ALWAYS_USE_HTTPS, "on")
            .map_err(|error| refused(error, Permission::ZoneSettingsEdit))
    }
}

/// Gives the zone and the tunnel's token to the engine, which seals the token and hands it to
/// the agent's cloudflared. The API token stays here.
struct Ingress {
    cloudflare: Cloudflare,
    console: Rc<Console>,
}

impl Ingress {
    /// Waits for the engine to answer on `admin.sock`, `ENGINE_TIMEOUT` at most, and returns
    /// whether it holds this zone and `tunnel`.
    fn engine_holds(&self, host: &Host, tunnel: &TunnelInfo) -> anyhow::Result<bool> {
        let start = self.console.now();
        loop {
            match host.engine_ingress() {
                Ok(ingress) => {
                    return Ok(ingress.is_some_and(|ingress| {
                        ingress.zone == self.cloudflare.site.zone.as_str()
                            && ingress.tunnel_id == tunnel.id
                    }));
                }
                Err(error)
                    if is_refusal(&error)
                        || self.console.now().saturating_sub(start) >= ENGINE_TIMEOUT =>
                {
                    return Err(error.context(
                        "The engine does not answer. Check its log: journalctl -u brume-engine",
                    ));
                }
                Err(_) => self.console.sleep(ENGINE_POLL),
            }
        }
    }
}

impl Step for Ingress {
    fn name(&self) -> &str {
        "Give the tunnel to the engine, which runs cloudflared"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        let Cloudflare { client, site } = &self.cloudflare;
        let Some(tunnel) = find_tunnel(client, site)? else {
            return Ok(false);
        };
        // An engine that does not run yet does not hold it.
        Ok(host.engine_ingress().is_ok_and(|ingress| {
            ingress.is_some_and(|ingress| {
                ingress.zone == site.zone.as_str() && ingress.tunnel_id == tunnel.id
            })
        }))
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        let Cloudflare { client, site } = &self.cloudflare;
        let tunnel = tunnel(&self.cloudflare)?;
        if self.engine_holds(host, &tunnel)? {
            return Ok(());
        }
        let tunnel_token = client
            .tunnel_token(&site.account_id, &tunnel.id)
            .map_err(|error| refused(error, Permission::TunnelEdit))?;
        let request = AdminRequest::ConfigureIngress {
            zone: site.zone.to_string(),
            tunnel_id: tunnel.id,
            tunnel_token,
        };
        match host.admin_change(&request)? {
            AdminReply::IngressConfigured => Ok(()),
            _ => bail!("The engine answered with something other than the public entry."),
        }
    }
}

#[cfg(test)]
mod tests;
