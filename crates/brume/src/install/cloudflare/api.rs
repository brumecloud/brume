//! The calls that `brume install` makes to Cloudflare's API v4, blocking.
//!
//! The token travels in a header marked sensitive, and nothing else holds it: the client's
//! `Debug` masks it, and Cloudflare's messages are scrubbed of it before an error quotes them.
//! An answer that does not parse is reported without its contents: the tunnel token comes in
//! one.

use std::fmt;
use std::io::Read as _;
use std::time::Duration;

use anyhow::Context as _;
use reqwest::Method;
use reqwest::blocking::Client as Http;
use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// Cloudflare's API.
pub const API: &str = "https://api.cloudflare.com/client/v4";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The largest answer read: Brume's queries return a few records.
const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

/// The most of Cloudflare's messages that an error quotes.
const MAX_MESSAGE_CHARS: usize = 1024;

/// Cloudflare's error codes for a token that may not make the call: "Authentication error",
/// and "Unauthorized to access requested resource".
const DENIED_CODES: [i64; 2] = [10000, 9109];

/// A zone, as `GET /zones` lists it.
#[derive(Clone, Debug, Deserialize)]
pub struct ZoneInfo {
    pub id: String,
    pub name: String,
    /// `active`, or `pending` until the registrar points to Cloudflare's name servers.
    #[serde(default)]
    pub status: String,
    pub account: AccountRef,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountRef {
    pub id: String,
}

/// A Cloudflare tunnel.
#[derive(Clone, Debug, Deserialize)]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
}

/// A DNS record.
#[derive(Clone, Debug, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub proxied: bool,
}

/// A call that failed: Cloudflare refused it, or could not be reached. Never holds the token.
#[derive(Debug)]
pub struct ApiError {
    /// The method and the path, such as `GET /zones`.
    call: String,
    /// The HTTP status, if Cloudflare answered.
    status: Option<u16>,
    /// Cloudflare's error codes.
    codes: Vec<i64>,
    /// Cloudflare's messages, or why the call failed.
    message: String,
}

impl ApiError {
    /// Whether Cloudflare answered, rather than could not be reached.
    pub fn answered(&self) -> bool {
        self.status.is_some()
    }

    /// Whether the token may not make the call: a permission is missing.
    pub fn denied(&self) -> bool {
        matches!(self.status, Some(401 | 403))
            || self.codes.iter().any(|code| DENIED_CODES.contains(code))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Some(status) => write!(
                f,
                "Cloudflare answered {} with HTTP {status}: {}",
                self.call, self.message
            ),
            None => write!(
                f,
                "Cannot call Cloudflare ({}): {}",
                self.call, self.message
            ),
        }
    }
}

impl std::error::Error for ApiError {}

/// A client of Cloudflare's API, with the admin's token.
pub struct Client {
    http: Http,
    base: String,
    token: SecretString,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("base", &self.base)
            .field("token", &"<redacted>")
            .finish()
    }
}

impl Client {
    /// Cloudflare's API, over HTTPS only.
    pub fn new(token: SecretString) -> anyhow::Result<Self> {
        Self::build(API, token, true)
    }

    /// A fake API at `base`, over plain HTTP to 127.0.0.1.
    #[cfg(test)]
    pub fn local(base: &str, token: SecretString) -> Self {
        Self::build(base, token, false).expect("a client")
    }

    fn build(base: &str, token: SecretString, https_only: bool) -> anyhow::Result<Self> {
        // rustls takes ring's cryptography; another provider installed first is kept.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let mut builder = Http::builder()
            .user_agent(concat!("brume-install/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .https_only(https_only);
        if !https_only {
            builder = builder.no_proxy();
        }
        let http = builder
            .build()
            .context("cannot set up the HTTPS client for Cloudflare")?;
        Ok(Self {
            http,
            base: base.trim_end_matches('/').to_owned(),
            token,
        })
    }

    /// The zones named `name` that the token can read.
    pub fn zones(&self, name: &str) -> Result<Vec<ZoneInfo>, ApiError> {
        self.get("/zones", &[("name", name), ("per_page", "50")])
    }

    /// The tunnels of `account` named `name`, deleted ones excluded.
    pub fn tunnels(&self, account: &str, name: &str) -> Result<Vec<TunnelInfo>, ApiError> {
        let path = format!("/accounts/{}/cfd_tunnel", segment(account)?);
        self.get(&path, &[("name", name), ("is_deleted", "false")])
    }

    /// Creates the tunnel `name` in `account`, configured locally: its routes come from
    /// cloudflared's command line, never from Cloudflare's dashboard.
    pub fn create_tunnel(&self, account: &str, name: &str) -> Result<TunnelInfo, ApiError> {
        let path = format!("/accounts/{}/cfd_tunnel", segment(account)?);
        let body = json!({ "name": name, "config_src": "local" });
        self.call_as(Method::POST, &path, &[], Some(&body))
    }

    /// The token that cloudflared runs the tunnel `id` with.
    pub fn tunnel_token(&self, account: &str, id: &str) -> Result<SecretString, ApiError> {
        let path = format!(
            "/accounts/{}/cfd_tunnel/{}/token",
            segment(account)?,
            segment(id)?
        );
        let call = format!("GET {path}");
        match self.call(Method::GET, &path, &[], None)? {
            Value::String(token) => Ok(SecretString::from(token)),
            _ => Err(self.error(&call, Some(200), &[], "the answer holds no token")),
        }
    }

    /// The DNS records of `zone` named `name`.
    pub fn records(&self, zone: &str, name: &str) -> Result<Vec<DnsRecord>, ApiError> {
        let path = format!("/zones/{}/dns_records", segment(zone)?);
        self.get(&path, &[("name", name), ("per_page", "100")])
    }

    /// Creates the proxied CNAME `name` → `target` in `zone`.
    pub fn create_cname(&self, zone: &str, name: &str, target: &str) -> Result<(), ApiError> {
        let path = format!("/zones/{}/dns_records", segment(zone)?);
        let body = json!({
            "type": "CNAME",
            "name": name,
            "content": target,
            "proxied": true,
            "ttl": 1,
            "comment": "Brume's public entry, created by brume install",
        });
        self.call(Method::POST, &path, &[], Some(&body)).map(drop)
    }

    /// Makes the record `id` of `zone` proxied.
    pub fn proxy_record(&self, zone: &str, id: &str) -> Result<(), ApiError> {
        let path = format!("/zones/{}/dns_records/{}", segment(zone)?, segment(id)?);
        let body = json!({ "proxied": true });
        self.call(Method::PATCH, &path, &[], Some(&body)).map(drop)
    }

    /// The value of the setting `id` of `zone`, such as `on`.
    pub fn setting(&self, zone: &str, id: &str) -> Result<String, ApiError> {
        let path = format!("/zones/{}/settings/{}", segment(zone)?, segment(id)?);
        let call = format!("GET {path}");
        let result = self.call(Method::GET, &path, &[], None)?;
        match result.get("value") {
            Some(Value::String(value)) => Ok(value.clone()),
            _ => Err(self.error(&call, Some(200), &[], "the answer holds no value")),
        }
    }

    /// Sets the setting `id` of `zone` to `value`.
    pub fn set_setting(&self, zone: &str, id: &str, value: &str) -> Result<(), ApiError> {
        let path = format!("/zones/{}/settings/{}", segment(zone)?, segment(id)?);
        let body = json!({ "value": value });
        self.call(Method::PATCH, &path, &[], Some(&body)).map(drop)
    }

    /// Whether the token may make a call that changes nothing: a write that Cloudflare refuses
    /// as invalid, or one that sets what is already set. `Ok(false)` if a permission is
    /// missing.
    pub fn may(&self, method: Method, path: &str, body: &Value) -> Result<bool, ApiError> {
        let call = format!("{method} {path}");
        let (status, envelope) = self.send(method, path, &[], Some(body))?;
        let error = self.api_error(&call, status, &envelope);
        match status {
            _ if error.denied() => Ok(false),
            200..=499 => Ok(true),
            _ => Err(error),
        }
    }

    fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T, ApiError> {
        self.call_as(Method::GET, path, query, None)
    }

    fn call_as<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&Value>,
    ) -> Result<T, ApiError> {
        let call = format!("{method} {path}");
        let result = self.call(method, path, query, body)?;
        serde_json::from_value(result).map_err(|_| {
            self.error(
                &call,
                Some(200),
                &[],
                "the answer is not what Brume expects",
            )
        })
    }

    /// The `result` of a successful call.
    fn call(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&Value>,
    ) -> Result<Value, ApiError> {
        let call = format!("{method} {path}");
        let (status, mut envelope) = self.send(method, path, query, body)?;
        let success = envelope.get("success").and_then(Value::as_bool) == Some(true);
        if !(200..300).contains(&status) || !success {
            return Err(self.api_error(&call, status, &envelope));
        }
        Ok(envelope
            .get_mut("result")
            .map(Value::take)
            .unwrap_or(Value::Null))
    }

    /// Sends a call, and returns the HTTP status and the JSON answer.
    fn send(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&Value>,
    ) -> Result<(u16, Value), ApiError> {
        let call = format!("{method} {path}");
        let mut url = reqwest::Url::parse(&format!("{}{path}", self.base))
            .map_err(|_| self.error(&call, None, &[], "invalid URL"))?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        // `bearer_auth` marks the header sensitive.
        let mut request = self
            .http
            .request(method, url)
            .bearer_auth(self.token.expose_secret());
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().map_err(|error| {
            let mut message = error.to_string();
            let mut source = std::error::Error::source(&error);
            while let Some(cause) = source {
                message = format!("{message}: {cause}");
                source = cause.source();
            }
            self.error(&call, None, &[], &message)
        })?;
        let status = response.status().as_u16();
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| self.error(&call, Some(status), &[], "cannot read the answer"))?;
        if bytes.len() as u64 > MAX_RESPONSE_BYTES {
            return Err(self.error(&call, Some(status), &[], "the answer is too large"));
        }
        let envelope = serde_json::from_slice(&bytes)
            .map_err(|_| self.error(&call, Some(status), &[], "the answer is not JSON"))?;
        Ok((status, envelope))
    }

    /// The error of a call that Cloudflare answered with `status` and `envelope`.
    fn api_error(&self, call: &str, status: u16, envelope: &Value) -> ApiError {
        let mut codes = Vec::new();
        let mut messages = Vec::new();
        let errors = envelope.get("errors").and_then(Value::as_array);
        for error in errors.into_iter().flatten() {
            let chain = error.get("error_chain").and_then(Value::as_array);
            for entry in std::iter::once(error).chain(chain.into_iter().flatten()) {
                let code = entry.get("code").and_then(Value::as_i64);
                let message = entry
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("no message");
                match code {
                    Some(code) => {
                        codes.push(code);
                        messages.push(format!("{message} [{code}]"));
                    }
                    None => messages.push(message.to_owned()),
                }
            }
        }
        let message = if messages.is_empty() {
            "no error message".to_owned()
        } else {
            messages.join("; ")
        };
        self.error(call, Some(status), &codes, &message)
    }

    /// An error, its message scrubbed of the token and bounded.
    fn error(&self, call: &str, status: Option<u16>, codes: &[i64], message: &str) -> ApiError {
        let token = self.token.expose_secret();
        let mut message = if token.is_empty() {
            message.to_owned()
        } else {
            message.replace(token, "<redacted>")
        };
        if let Some((cut, _)) = message.char_indices().nth(MAX_MESSAGE_CHARS) {
            message.truncate(cut);
            message.push('…');
        }
        ApiError {
            call: call.to_owned(),
            status,
            codes: codes.to_vec(),
            message,
        }
    }
}

/// `id`, if it can be a segment of a path: Cloudflare's ids are hexadecimal or UUIDs.
fn segment(id: &str) -> Result<&str, ApiError> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    if valid {
        Ok(id)
    } else {
        Err(ApiError {
            call: "a path".to_owned(),
            status: None,
            codes: Vec::new(),
            message: format!("{id:?} is not a Cloudflare id"),
        })
    }
}
