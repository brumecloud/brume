//! A fake of Cloudflare's API on 127.0.0.1, for the tests: an account with zones, tunnels,
//! DNS records and one zone setting, behind a token that lacks the permissions it is told to.
//! It answers as Cloudflare does: the same envelope, codes and statuses.

use std::collections::BTreeSet;
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{Value, json};

use super::Permission;

pub const ACCOUNT: &str = "480473627764c92e7e400be60990cc87";
pub const ZONE_ID: &str = "023e105f4ecef8ad9ca31a8372d0c353";
pub const API_TOKEN: &str = "cf-api-token-Zx81_s3cr3t";

/// A tunnel of the account.
#[derive(Clone, Debug)]
pub struct Tunnel {
    pub id: String,
    pub name: String,
    pub deleted: bool,
    pub token: String,
}

/// A DNS record of the zone.
#[derive(Clone, Debug)]
pub struct Record {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub content: String,
    pub proxied: bool,
}

/// The account, and what the token may do.
#[derive(Debug)]
pub struct State {
    /// The zone the account holds, and its status.
    pub zone: String,
    pub zone_status: String,
    pub tunnels: Vec<Tunnel>,
    pub records: Vec<Record>,
    pub always_use_https: String,
    /// What the token lacks.
    pub denied: BTreeSet<Permission>,
    /// Each request: method, path and query.
    pub requests: Vec<String>,
    next_id: u32,
}

impl State {
    pub fn new(zone: &str) -> Self {
        Self {
            zone: zone.to_owned(),
            zone_status: "active".to_owned(),
            tunnels: Vec::new(),
            records: Vec::new(),
            always_use_https: "off".to_owned(),
            denied: BTreeSet::new(),
            requests: Vec::new(),
            next_id: 1,
        }
    }

    /// A tunnel id, as Cloudflare's UUIDs.
    pub fn tunnel_id(n: u32) -> String {
        format!("c1744f8b-faa1-48a4-9e5c-{n:012x}")
    }

    /// Adds the tunnel `name`, as a previous install created it.
    pub fn add_tunnel(&mut self, name: &str) -> String {
        let id = Self::tunnel_id(self.next());
        self.tunnels.push(Tunnel {
            id: id.clone(),
            name: name.to_owned(),
            deleted: false,
            token: format!("eyJ0IjoiInt-tunnel-token-{id}"),
        });
        id
    }

    /// Adds a record to the zone.
    pub fn add_record(&mut self, kind: &str, name: &str, content: &str, proxied: bool) {
        let id = format!("rec{}", self.next());
        self.records.push(Record {
            id,
            kind: kind.to_owned(),
            name: name.to_owned(),
            content: content.to_owned(),
            proxied,
        });
    }

    fn next(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// The requests that would change the account if they succeeded, the probes included.
    pub fn writes(&self) -> Vec<String> {
        self.requests
            .iter()
            .filter(|request| !request.starts_with("GET "))
            .cloned()
            .collect()
    }
}

/// The fake API, on a port of 127.0.0.1, until the test ends.
pub struct FakeCloudflare {
    state: Arc<Mutex<State>>,
    base: String,
}

impl FakeCloudflare {
    pub fn start(state: State) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("binds");
        let base = format!(
            "http://{}/client/v4",
            listener.local_addr().expect("an address")
        );
        let state = Arc::new(Mutex::new(state));
        let shared = Arc::clone(&state);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                serve(stream, &shared);
            }
        });
        Self { state, base }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("not poisoned")
    }
}

/// Answers one request, then closes the connection.
fn serve(stream: TcpStream, state: &Mutex<State>) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let mut parts = line.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return;
    };
    let (method, target) = (method.to_owned(), target.to_owned());
    let mut length = 0;
    let mut authorization = String::new();
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() || header.trim().is_empty() {
            break;
        }
        let (name, value) = header.split_once(':').unwrap_or_default();
        match name.trim().to_ascii_lowercase().as_str() {
            "content-length" => length = value.trim().parse().unwrap_or(0),
            "authorization" => value.trim().clone_into(&mut authorization),
            _ => {}
        }
    }
    let mut body = vec![0; length];
    if reader.read_exact(&mut body).is_err() {
        return;
    }
    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    let url = reqwest::Url::parse(&format!("http://fake{target}")).expect("a URL");
    let path = url
        .path()
        .strip_prefix("/client/v4")
        .unwrap_or("")
        .to_owned();
    let query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
    let (status, answer) = {
        let mut state = state.lock().expect("not poisoned");
        let shown = match url.query() {
            Some(query) => format!("{method} {path}?{query}"),
            None => format!("{method} {path}"),
        };
        state.requests.push(shown);
        if authorization == format!("Bearer {API_TOKEN}") {
            route(&mut state, &method, &path, &query, &body)
        } else {
            // Echoes what it got, as a careless server might: the client must scrub it.
            failure(403, 9109, &format!("Invalid access token: {authorization}"))
        }
    };
    let text = answer.to_string();
    let mut stream = reader.into_inner();
    let _ = write!(
        stream,
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{text}",
        text.len()
    );
}

fn success(result: Value) -> (u16, Value) {
    (
        200,
        json!({ "success": true, "errors": [], "messages": [], "result": result }),
    )
}

fn failure(status: u16, code: i64, message: &str) -> (u16, Value) {
    (
        status,
        json!({
            "success": false,
            "errors": [{ "code": code, "message": message }],
            "messages": [],
            "result": null,
        }),
    )
}

fn denied() -> (u16, Value) {
    failure(403, 10000, "Authentication error")
}

fn param<'a>(query: &'a [(String, String)], name: &str) -> Option<&'a str> {
    query
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn tunnel_json(tunnel: &Tunnel) -> Value {
    json!({
        "id": tunnel.id,
        "name": tunnel.name,
        "deleted_at": if tunnel.deleted { json!("2026-10-01T00:00:00Z") } else { Value::Null },
        "status": "inactive",
        "remote_config": false,
    })
}

fn record_json(record: &Record) -> Value {
    json!({
        "id": record.id,
        "type": record.kind,
        "name": record.name,
        "content": record.content,
        "proxied": record.proxied,
        "ttl": 1,
    })
}

fn route(
    state: &mut State,
    method: &str,
    path: &str,
    query: &[(String, String)],
    body: &Value,
) -> (u16, Value) {
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    let has = |permission| !state.denied.contains(&permission);
    match (method, segments.as_slice()) {
        // Without Zone Read, the zone is not listed.
        ("GET", ["zones"]) => {
            let listed = has(Permission::ZoneRead) && param(query, "name") == Some(&state.zone);
            let zones: Vec<Value> = listed
                .then(|| {
                    json!({
                        "id": ZONE_ID,
                        "name": state.zone,
                        "status": state.zone_status,
                        "account": { "id": ACCOUNT, "name": "p6n" },
                    })
                })
                .into_iter()
                .collect();
            success(json!(zones))
        }
        (_, ["accounts", ACCOUNT, "cfd_tunnel", ..]) if !has(Permission::TunnelEdit) => denied(),
        ("GET", ["accounts", ACCOUNT, "cfd_tunnel"]) => {
            let name = param(query, "name");
            let live = param(query, "is_deleted") == Some("false");
            let tunnels: Vec<Value> = state
                .tunnels
                .iter()
                .filter(|tunnel| name.is_none_or(|name| tunnel.name == name))
                .filter(|tunnel| !(live && tunnel.deleted))
                .map(tunnel_json)
                .collect();
            success(json!(tunnels))
        }
        ("POST", ["accounts", ACCOUNT, "cfd_tunnel"]) => {
            let name = body.get("name").and_then(Value::as_str).unwrap_or("");
            if name.is_empty() {
                return failure(400, 1006, "Tunnel name can't be empty");
            }
            if body.get("config_src").and_then(Value::as_str) != Some("local") {
                return failure(400, 1001, "Unexpected config_src");
            }
            let exists = state
                .tunnels
                .iter()
                .any(|tunnel| tunnel.name == name && !tunnel.deleted);
            if exists {
                return failure(409, 1013, "You already have a tunnel with this name");
            }
            let id = state.add_tunnel(name);
            let tunnel = state.tunnels.iter().find(|tunnel| tunnel.id == id);
            success(tunnel.map(tunnel_json).unwrap_or(Value::Null))
        }
        ("GET", ["accounts", ACCOUNT, "cfd_tunnel", id, "token"]) => {
            match state.tunnels.iter().find(|tunnel| tunnel.id == *id) {
                Some(tunnel) => success(json!(tunnel.token)),
                None => failure(404, 1003, "Tunnel not found"),
            }
        }
        (_, ["zones", ZONE_ID, "dns_records", ..]) if !has(Permission::DnsEdit) => denied(),
        ("GET", ["zones", ZONE_ID, "dns_records"]) => {
            let name = param(query, "name");
            let records: Vec<Value> = state
                .records
                .iter()
                .filter(|record| name.is_none_or(|name| record.name == name))
                .map(record_json)
                .collect();
            success(json!(records))
        }
        ("POST", ["zones", ZONE_ID, "dns_records", "batch"]) => {
            failure(400, 9207, "Request body is invalid.")
        }
        ("POST", ["zones", ZONE_ID, "dns_records"]) => {
            let field = |key: &str| body.get(key).and_then(Value::as_str).unwrap_or("");
            let taken = state
                .records
                .iter()
                .any(|record| record.name == field("name"));
            if taken {
                return failure(
                    400,
                    81053,
                    "An A, AAAA, or CNAME record with that host already exists.",
                );
            }
            let proxied = body.get("proxied").and_then(Value::as_bool) == Some(true);
            state.add_record(field("type"), field("name"), field("content"), proxied);
            success(state.records.last().map(record_json).unwrap_or(Value::Null))
        }
        ("PATCH", ["zones", ZONE_ID, "dns_records", id]) => {
            let Some(record) = state.records.iter_mut().find(|record| record.id == *id) else {
                return failure(404, 81044, "Record does not exist.");
            };
            if let Some(proxied) = body.get("proxied").and_then(Value::as_bool) {
                record.proxied = proxied;
            }
            success(record_json(record))
        }
        (_, ["zones", ZONE_ID, "settings", "always_use_https"])
            if !has(Permission::ZoneSettingsEdit) =>
        {
            denied()
        }
        ("GET", ["zones", ZONE_ID, "settings", "always_use_https"]) => success(json!({
            "id": "always_use_https",
            "value": state.always_use_https,
            "editable": true,
        })),
        ("PATCH", ["zones", ZONE_ID, "settings", "always_use_https"]) => {
            match body.get("value").and_then(Value::as_str) {
                Some(value @ ("on" | "off")) => {
                    value.clone_into(&mut state.always_use_https);
                    success(json!({ "id": "always_use_https", "value": value }))
                }
                _ => failure(400, 1007, "Invalid value for zone setting always_use_https"),
            }
        }
        _ => failure(404, 7003, "Could not route to the path"),
    }
}
