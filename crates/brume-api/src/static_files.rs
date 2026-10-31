//! The panel, embedded at build time by `build.rs`: the files of `web/dist`, or a page that
//! says how to build it.

use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};

use crate::routes::json_error;

include!(concat!(env!("OUT_DIR"), "/panel.rs"));

/// Serves the file at the request's path, or `index.html` for any other path: the panel routes
/// in the browser.
pub(crate) async fn serve(method: Method, uri: Uri) -> Response {
    if !matches!(method, Method::GET | Method::HEAD) {
        return json_error(StatusCode::METHOD_NOT_ALLOWED, "Only GET reads the panel.");
    }
    let path = uri.path().trim_start_matches('/');
    let (name, bytes) = file(path)
        .or_else(|| file("index.html"))
        .expect("build.rs always embeds index.html");
    let cache = if name.starts_with("assets/") {
        // Vite puts a content hash in the name of each asset.
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [
            (CONTENT_TYPE, HeaderValue::from_static(content_type(name))),
            (CACHE_CONTROL, HeaderValue::from_static(cache)),
        ],
        *bytes,
    )
        .into_response()
}

fn file(path: &str) -> Option<&'static (&'static str, &'static [u8])> {
    PANEL_FILES.iter().find(|(name, _)| *name == path)
}

fn content_type(name: &str) -> &'static str {
    let extension = name.rsplit_once('.').map_or("", |(_, extension)| extension);
    match extension {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "webmanifest" => "application/manifest+json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
