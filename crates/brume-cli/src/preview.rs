use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use brume_core::{BASE_PATH_PLACEHOLDER, BundleManifest, validate_relative_path};

use crate::embedded::{WEB_RUNTIME, WEB_THEME};

struct PreviewState {
    output: PathBuf,
    pages: HashMap<String, String>,
    title: String,
}

pub async fn serve(
    output: PathBuf,
    manifest: BundleManifest,
    port: u16,
    open_browser: bool,
) -> Result<()> {
    let state = Arc::new(PreviewState {
        output,
        pages: manifest
            .pages
            .into_iter()
            .map(|page| (page.route, page.object_path))
            .collect(),
        title: manifest.title,
    });
    let router = Router::new()
        .route("/", get(root_page))
        .route("/_brume/runtime.js", get(runtime))
        .route("/_brume/theme.css", get(theme))
        .route("/_assets/{*path}", get(asset))
        .route("/{*path}", get(nested_page))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    let address = listener.local_addr()?;
    let url = format!("http://{address}");
    println!("Preview: {url}");
    if open_browser {
        open::that(&url).context("opening the preview in the default browser")?;
    }
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

async fn root_page(State(state): State<Arc<PreviewState>>) -> Response {
    page_response(state, "/").await
}

async fn nested_page(State(state): State<Arc<PreviewState>>, Path(path): Path<String>) -> Response {
    let route = format!("/{}", path.trim_end_matches('/'));
    page_response(state, &route).await
}

async fn page_response(state: Arc<PreviewState>, route: &str) -> Response {
    let Some(object_path) = state.pages.get(route) else {
        return (StatusCode::NOT_FOUND, "Plan page not found").into_response();
    };
    let path = state.output.join(object_path);
    match fs::read_to_string(&path) {
        Ok(fragment) => Html(shell(
            &state.title,
            &fragment.replace(BASE_PATH_PLACEHOLDER, ""),
        ))
        .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot read {}: {error}", path.display()),
        )
            .into_response(),
    }
}

async fn asset(State(state): State<Arc<PreviewState>>, Path(path): Path<String>) -> Response {
    if validate_relative_path(&path).is_err() {
        return (StatusCode::BAD_REQUEST, "Invalid asset path").into_response();
    }
    let file = state.output.join("assets").join(&path);
    match fs::read(&file) {
        Ok(bytes) => (
            [(
                header::CONTENT_TYPE,
                mime_guess::from_path(&file)
                    .first_or_octet_stream()
                    .as_ref(),
            )],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

async fn runtime() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        WEB_RUNTIME,
    )
}

async fn theme() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        WEB_THEME,
    )
}

fn shell(title: &str, fragment: &str) -> String {
    let title = escape_html(title);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title}</title><link rel=\"stylesheet\" href=\"/_brume/theme.css\"><script type=\"module\" src=\"/_brume/runtime.js\"></script></head><body><div class=\"brume-shell\"><header class=\"brume-topbar\"><a href=\"/\">Brume</a><button class=\"brume-theme-toggle\" data-brume-theme-toggle type=\"button\">Theme</button></header>{fragment}</div></body></html>"
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
