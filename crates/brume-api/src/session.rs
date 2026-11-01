//! The `brume_session` cookie: set from the engine's session token at login, read on every
//! call and passed to the engine, which checks it. Its value is never logged.

use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue};

/// The name of the session cookie.
const COOKIE_NAME: &str = "brume_session";

/// The longest token accepted from a cookie. Engine tokens are far shorter.
const MAX_TOKEN_LEN: usize = 512;

/// The session token of the request's `brume_session` cookie, if it has one.
pub(crate) fn token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, token)| token)
        .filter(|token| is_cookie_safe(token))
        .map(str::to_owned)
}

/// Whether the browser may send the session cookie over plain HTTP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CookieSecurity {
    /// `brume dev`, or a host reached by an SSH tunnel: plain HTTP on localhost.
    AnyScheme,
    /// A host behind `tailscale serve`: `Secure`, HTTPS only.
    HttpsOnly,
}

impl CookieSecurity {
    fn attribute(self) -> &'static str {
        match self {
            Self::AnyScheme => "",
            Self::HttpsOnly => "; Secure",
        }
    }
}

/// The `Set-Cookie` that stores `token`. `None` if the token holds a character that would
/// break out of the cookie value.
pub(crate) fn set_cookie(
    token: &str,
    security: CookieSecurity,
) -> Option<(axum::http::HeaderName, HeaderValue)> {
    if !is_cookie_safe(token) {
        return None;
    }
    let value = format!(
        "{COOKIE_NAME}={token}; HttpOnly; SameSite=Strict; Path=/{}",
        security.attribute()
    );
    HeaderValue::from_str(&value)
        .ok()
        .map(|value| (SET_COOKIE, value))
}

/// The `Set-Cookie` that deletes the session cookie.
pub(crate) fn clear_cookie(security: CookieSecurity) -> (axum::http::HeaderName, HeaderValue) {
    let value = format!(
        "{COOKIE_NAME}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0{}",
        security.attribute()
    );
    (
        SET_COOKIE,
        HeaderValue::from_str(&value).expect("the cookie is ASCII"),
    )
}

/// Letters, digits and `-._~`: what the engine's tokens are made of.
fn is_cookie_safe(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= MAX_TOKEN_LEN
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'))
}

#[cfg(test)]
mod tests {
    use axum::http::header::COOKIE;
    use axum::http::{HeaderMap, HeaderValue};

    use super::{CookieSecurity, set_cookie, token};

    #[test]
    fn reads_the_session_among_other_cookies() {
        let mut headers = HeaderMap::new();
        headers.append(COOKIE, HeaderValue::from_static("theme=dark"));
        headers.append(
            COOKIE,
            HeaderValue::from_static("a=b; brume_session=abc-123"),
        );
        assert_eq!(token(&headers).as_deref(), Some("abc-123"));
    }

    #[test]
    fn refuses_tokens_that_break_the_cookie() {
        for security in [CookieSecurity::AnyScheme, CookieSecurity::HttpsOnly] {
            assert!(set_cookie("abc; Domain=evil.example", security).is_none());
            assert!(set_cookie("", security).is_none());
        }
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_static("brume_session=a\"b"));
        assert_eq!(token(&headers), None);
    }
}
