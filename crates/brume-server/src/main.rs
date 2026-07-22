mod auth;
mod config;
mod deployments;
mod error;
mod gc;
mod plans;
mod public;
mod state;
mod storage;
mod tunnels;
mod util;
mod web;

mod build_metadata {
    include!(concat!(env!("OUT_DIR"), "/build_metadata.rs"));
}

use std::time::Duration;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::HOST},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::{Parser, Subcommand};
use config::Config;
use serde::Serialize;
use state::AppState;
use tower::ServiceExt;
use tower_cookies::CookieManagerLayer;
use tower_http::{catch_panic::CatchPanicLayer, compression::CompressionLayer, trace::TraceLayer};

#[derive(Parser)]
#[command(name = "brume-server", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Serve,
    GarbageCollect,
    CreateDevToken {
        #[arg(long)]
        github_id: i64,
        #[arg(long)]
        login: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "brume=info,tower_http=info".into()),
        )
        .json()
        .init();
    let cli = Cli::parse();
    let config = Config::from_env()?;
    let bind = config.bind;
    let state = AppState::initialize(config).await?;
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(state, bind).await,
        Command::GarbageCollect => gc::run(&state).await,
        Command::CreateDevToken { github_id, login } => {
            let user = auth::provision_user(&state, github_id, &login).await?;
            let credentials = auth::issue_development_credentials(&state, user.id).await?;
            println!("{}", serde_json::to_string(&credentials)?);
            Ok(())
        }
    }
}

async fn serve(state: AppState, bind: std::net::SocketAddr) -> Result<()> {
    let router = application_router(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(address = %listener.local_addr()?, "Brume server listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await?;
    Ok(())
}

fn application_router(state: AppState) -> Router {
    Router::new()
        .fallback(dispatch_by_host)
        .layer(CatchPanicLayer::new())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(state)
}

async fn dispatch_by_host(State(state): State<AppState>, request: Request) -> Response {
    if host_matches(request.headers(), &state.config.api_public_host) {
        return Router::new()
            .route("/health", get(health))
            .merge(auth::api_router())
            .merge(deployments::api_router())
            .merge(plans::router())
            .merge(tunnels::router())
            .with_state(state)
            .oneshot(request)
            .await
            .expect("API router is infallible");
    }
    if host_matches(request.headers(), &state.config.auth_public_host) {
        return auth::browser_router()
            .with_state(state)
            .oneshot(request)
            .await
            .expect("auth router is infallible");
    }
    if host_matches(request.headers(), &state.config.plan_public_host) {
        return Router::new()
            .merge(auth::plan_router())
            .merge(web::router())
            .with_state(state)
            .oneshot(request)
            .await
            .expect("plan router is infallible");
    }
    let Some(label) = dynamic_public_label(request.headers(), &state.config) else {
        if request.uri().path() == "/health" {
            return health(State(state)).await.into_response();
        }
        return StatusCode::NOT_FOUND.into_response();
    };
    public::serve(state, label, request).await
}

fn host_matches(headers: &HeaderMap, expected: &str) -> bool {
    headers
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

fn dynamic_public_label(headers: &HeaderMap, config: &Config) -> Option<String> {
    let mut host = headers.get(HOST)?.to_str().ok()?.to_ascii_lowercase();
    if host_matches(headers, &config.api_public_host)
        || host_matches(headers, &config.auth_public_host)
        || host_matches(headers, &config.plan_public_host)
    {
        return None;
    }
    if let Some(port) = config.public_port
        && !matches!(
            (config.public_scheme.as_str(), port),
            ("http", 80) | ("https", 443)
        )
    {
        host = host.strip_suffix(&format!(":{port}"))?.to_owned();
    } else if host.contains(':') {
        return None;
    }
    let label = host.strip_suffix(&format!(".{}", config.public_domain))?;
    if label.is_empty()
        || label.len() > 63
        || label.contains('.')
        || label.starts_with('-')
        || label.ends_with('-')
        || !label
            .chars()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-')
    {
        return None;
    }
    Some(label.to_owned())
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    commit: &'static str,
    commit_title: &'static str,
    commit_message: &'static str,
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let (status_code, status) = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.database)
        .await
    {
        Ok(1) => (StatusCode::OK, "ok"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, "database unavailable"),
    };
    (
        status_code,
        Json(HealthResponse {
            status,
            commit: build_metadata::COMMIT_SHA,
            commit_title: build_metadata::COMMIT_TITLE,
            commit_message: build_metadata::COMMIT_MESSAGE,
        }),
    )
}
