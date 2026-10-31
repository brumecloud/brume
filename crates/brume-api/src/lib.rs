//! The API service. It serves the HTTP API and the embedded panel on localhost, checks
//! hosts, origins and sessions, and forwards each call to the engine.

#![forbid(unsafe_code)]

mod csrf;
#[cfg(test)]
mod dto_ts;
mod engine_client;
pub mod framed;
pub mod paths;
mod routes;
mod session;
mod sse;
mod static_files;

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context as _;
use axum::Router;
use axum::extract::DefaultBodyLimit;

use crate::engine_client::EngineClient;
use crate::paths::Paths;

/// The port of the panel's Vite dev server, which proxies `/api` here.
const VITE_DEV_PORT: u16 = 5173;

/// How the API listens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiConfig {
    /// The port on 127.0.0.1.
    pub port: u16,
    /// The `Host` values accepted, such as `127.0.0.1:8080`. Any other gets a 421, which
    /// defeats DNS rebinding.
    pub allowed_hosts: Vec<String>,
}

impl ApiConfig {
    /// Listens on `port`, for the panel served here and for the Vite dev server.
    pub fn new(port: u16) -> Self {
        let allowed_hosts = [port, VITE_DEV_PORT]
            .into_iter()
            .flat_map(|port| [format!("127.0.0.1:{port}"), format!("localhost:{port}")])
            .collect();
        Self {
            port,
            allowed_hosts,
        }
    }
}

/// The state that every request shares.
pub(crate) struct AppState {
    engine: EngineClient,
    allowed_hosts: Vec<String>,
}

/// The routes of the API and the panel, behind the checks of `csrf`, calling the engine on
/// `<home>/run/api.sock`.
pub fn app(home: PathBuf, config: &ApiConfig) -> Router {
    let state = Arc::new(AppState {
        engine: EngineClient::new(Paths::new(home).api_socket()),
        allowed_hosts: config.allowed_hosts.clone(),
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
pub async fn run(home: PathBuf, config: ApiConfig) -> anyhow::Result<()> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, config.port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("cannot listen on {address}"))?;
    tracing::info!(%address, "the API is listening");
    axum::serve(listener, app(home, &config))
        .await
        .context("the API server stopped")
}
