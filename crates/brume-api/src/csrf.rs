//! The checks run before every request, and the security headers set on every response. See
//! `archi/base/04-panel-admin.md` (CSRF, XSS): the API listens on 127.0.0.1 only, accepts
//! known hosts only, refuses cross-site requests, requires `X-Brume` on the API, checks
//! `Origin` on writes, and sends no CORS header.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::{
    CACHE_CONTROL, CONTENT_LENGTH, CONTENT_SECURITY_POLICY, CONTENT_TYPE, HOST, ORIGIN,
    REFERRER_POLICY, TRANSFER_ENCODING, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;
use crate::routes::json_error;

/// The largest request body, in bytes.
pub(crate) const MAX_BODY_BYTES: usize = 64 * 1024;

/// The panel's Content Security Policy: no inline script, no framing, Trusted Types.
const CSP: &str = "default-src 'self'; base-uri 'none'; form-action 'self'; \
                   frame-ancestors 'none'; require-trusted-types-for 'script'";

/// Runs the checks, then the route, then sets the security headers.
pub(crate) async fn guard(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let api = is_api(request.uri().path());
    let mut response = match check(&state.allowed_hosts, &request, api) {
        Ok(()) => next.run(request).await,
        Err(Refusal(status, message)) => json_error(status, message),
    };
    let headers = response.headers_mut();
    headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(CSP));
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    if api {
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

/// Why a request is refused: its status and message.
struct Refusal(StatusCode, &'static str);

/// The refusal for a request that fails a check.
fn check(allowed_hosts: &[String], request: &Request, api: bool) -> Result<(), Refusal> {
    let headers = request.headers();
    // Exactly one `Host`, and a known one.
    let mut hosts = headers.get_all(HOST).iter();
    let host = match (hosts.next(), hosts.next()) {
        (Some(host), None) => host.to_str().ok(),
        _ => None,
    };
    if !host.is_some_and(|host| allowed_hosts.iter().any(|a| a.eq_ignore_ascii_case(host))) {
        return Err(Refusal(
            StatusCode::MISDIRECTED_REQUEST,
            "This host is not served here.",
        ));
    }
    // `none` is a typed address, a bookmark, or the login link opened from the terminal: it
    // may open the panel, but never call the API.
    //
    // Accepted deviation: a request with no `Sec-Fetch-Site` at all passes (`all` of nothing is
    // true). A browser always sends it; a client that is not a browser, such as the e2e tests,
    // does not, and CSRF is a browser attack. `X-Brume` below still covers a browser that would
    // not send it.
    let same_origin = headers
        .get_all("sec-fetch-site")
        .iter()
        .all(|site| match site.as_bytes() {
            b"same-origin" => true,
            b"none" => !api,
            _ => false,
        });
    if !same_origin {
        return Err(Refusal(
            StatusCode::FORBIDDEN,
            "Cross-site requests are refused.",
        ));
    }
    // EventSource cannot set a header: the log stream, a GET without side effect, relies on
    // Fetch Metadata alone.
    let log_stream = request.method() == Method::GET && is_log_stream(request.uri().path());
    if api && !log_stream && headers.get("x-brume").map(HeaderValue::as_bytes) != Some(b"1") {
        return Err(Refusal(
            StatusCode::FORBIDDEN,
            "The X-Brume header is missing.",
        ));
    }
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        // Accepted deviation, as for `Sec-Fetch-Site`: a write with no `Origin` passes. A
        // browser sends `Origin` on every write, a client that is not a browser does not.
        let known_origin = |origin: &HeaderValue| {
            allowed_hosts
                .iter()
                .any(|host| origin.as_bytes() == format!("http://{host}").as_bytes())
        };
        if !headers.get_all(ORIGIN).iter().all(known_origin) {
            return Err(Refusal(
                StatusCode::FORBIDDEN,
                "Requests from another origin are refused.",
            ));
        }
        if has_body(headers) && !is_json(headers) {
            return Err(Refusal(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "The request body must be JSON.",
            ));
        }
    }
    let too_long = headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > MAX_BODY_BYTES as u64);
    if too_long {
        return Err(Refusal(StatusCode::PAYLOAD_TOO_LARGE, BODY_TOO_LARGE));
    }
    Ok(())
}

/// The message for a body over `MAX_BODY_BYTES`.
pub(crate) const BODY_TOO_LARGE: &str = "The request body is larger than 64 KiB.";

fn is_api(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

/// `/api/environments/:id/logs`.
fn is_log_stream(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').collect();
    matches!(segments.as_slice(), ["", "api", "environments", id, "logs"] if !id.is_empty())
}

fn has_body(headers: &HeaderMap) -> bool {
    let length = headers
        .get(CONTENT_LENGTH)
        .is_some_and(|value| value.as_bytes() != b"0");
    length || headers.contains_key(TRANSFER_ENCODING)
}

fn is_json(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
}
