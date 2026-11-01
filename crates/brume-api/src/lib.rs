//! The API service. It serves the HTTP API and the embedded panel on localhost, checks
//! hosts, Tailscale identities, origins and sessions, and forwards each call to the engine.

#![forbid(unsafe_code)]

mod config;
mod csrf;
#[cfg(test)]
mod dto_ts;
mod engine_client;
mod routes;
mod session;
mod sse;
mod static_files;

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::Context as _;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use brume_runtime::paths::Paths;

pub use crate::config::{ApiConfig, ApiFileConfig, DEFAULT_PORT, SYSTEM_CONFIG_PATH};
use crate::engine_client::EngineClient;
use crate::session::CookieSecurity;

/// The state that every request shares.
pub(crate) struct AppState {
    engine: EngineClient,
    config: ApiConfig,
    cookie: CookieSecurity,
}

/// The routes of the API and the panel, behind the checks of `csrf`, calling the engine on
/// `api.sock`.
pub fn app(paths: &Paths, config: &ApiConfig) -> Router {
    let state = Arc::new(AppState {
        engine: EngineClient::new(paths.api_socket()),
        config: config.clone(),
        cookie: if config.secure_cookie {
            CookieSecurity::HttpsOnly
        } else {
            CookieSecurity::AnyScheme
        },
    });
    Router::new()
        .nest("/api", routes::api())
        .fallback(static_files::serve)
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            csrf::guard,
        ))
        .layer(DefaultBodyLimit::max(csrf::MAX_BODY_BYTES))
        .with_state(state)
}

/// Serves the API and the panel on `127.0.0.1:<port>` until the listener fails.
pub async fn run(paths: Paths, config: ApiConfig) -> anyhow::Result<()> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, config.port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("cannot listen on {address}"))?;
    tracing::info!(%address, "the API is listening");
    axum::serve(listener, app(&paths, &config))
        .await
        .context("the API server stopped")
}
