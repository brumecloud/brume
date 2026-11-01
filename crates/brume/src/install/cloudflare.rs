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
use std::fs::OpenOptions;
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

/// A permission of a Cloudflare API token, as the dashboard names it.
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
            Self::TunnelEdit => "Account → Cloudflare Tunnel → Edit",
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
    pub tunnel_name: String,
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

/// Asks for the token on the terminal, without echo. `stty` turns the echo off and on: no
/// unsafe code, and no crate.
fn prompt_token(zone: &Zone) -> anyhow::Result<SecretString> {
    let no_terminal = || {
        anyhow::anyhow!(
            "--domain needs a Cloudflare API token, and there is no terminal to type it. Set \
             {TOKEN_ENV}, and keep it through sudo: sudo --preserve-env={TOKEN_ENV} brume \
             install …, or curl … | sudo {TOKEN_ENV}=… bash -s -- …"
        )
    };
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
    let line = {
        let _hidden = Echo::off(&tty)?;
        let mut line = String::new();
        BufReader::new((&tty).take(MAX_TOKEN_BYTES as u64 + 2))
            .read_line(&mut line)
            .context("cannot read the token")?;
        line
    };
    writeln!(tty)?;
    token_from(&line)
}

/// The terminal's echo, off until dropped.
struct Echo {
    tty: std::fs::File,
}

impl Echo {
    fn off(tty: &std::fs::File) -> anyhow::Result<Self> {
        let echo = Self {
            tty: tty.try_clone()?,
        };
        if !echo.stty("-echo")? {
            bail!("Cannot hide the token as it is typed: `stty -echo` failed.");
        }
        Ok(echo)
    }

    fn stty(&self, setting: &str) -> anyhow::Result<bool> {
        let status = Command::new("stty")
            .arg(setting)
            .env_remove(TOKEN_ENV)
            .stdin(Stdio::from(self.tty.try_clone()?))
            .status()
            .context("cannot run stty")?;
        Ok(status.success())
    }
}

impl Drop for Echo {
    fn drop(&mut self) {
        let _ = self.stty("echo");
    }
}

/// The tunnel's name: `brume-`, the host name, and a hash of the machine id. Stable for this
/// server, so that a rerun finds its tunnel, and never another server's of the same name.
pub fn tunnel_name(host: &Host) -> anyhow::Result<String> {
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
    Ok(format!("brume-{label}-{}", hex::encode(&digest[..4])))
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

/// The live tunnel named `name`.
fn find_tunnel(client: &Client, site: &Site) -> Result<Option<TunnelInfo>, ApiError> {
    Ok(client
        .tunnels(&site.account_id, &site.tunnel_name)?
        .into_iter()
        .find(|tunnel| tunnel.name == site.tunnel_name && tunnel.deleted_at.is_none()))
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

/// Checks, before any change, that the token can set up `zone`, and that no other record
/// holds `*.<zone>`. Creates and changes nothing: the probes of the permissions to edit are
/// writes that Cloudflare refuses as invalid, or that set what is already set.
pub fn check(host: &Host, client: &Client, zone: &Zone) -> anyhow::Result<Site> {
    let tunnel_name = tunnel_name(host)?;
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
    let site = Site {
        zone: zone.clone(),
        zone_id: info.id,
        zone_status: info.status,
        account_id: info.account.id,
        tunnel_name,
    };

    let mut missing = BTreeSet::new();
    let tunnels_path = format!("/accounts/{}/cfd_tunnel", site.account_id);
    let tunnel = note(
        &mut missing,
        Permission::TunnelEdit,
        find_tunnel(client, &site),
    )?;
    // A tunnel without a name is invalid: Cloudflare refuses it as such if the token may
    // create tunnels.
    if tunnel.is_some() && !client.may(Method::POST, &tunnels_path, &json!({ "name": "" }))? {
        missing.insert(Permission::TunnelEdit);
    }
    let tunnel = tunnel.flatten();

    let records = note(
        &mut missing,
        Permission::DnsEdit,
        wildcard_records(client, &site),
    )?;
    // An empty batch of DNS changes is invalid.
    let batch = format!("/zones/{}/dns_records/batch", site.zone_id);
    if records.is_some() && !client.may(Method::POST, &batch, &json!({}))? {
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
        if !client.may(Method::PATCH, &path, &json!({ "value": value }))? {
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
