mod auth;
mod config;
mod error;
mod gc;
mod plans;
mod state;
mod storage;
mod util;
mod web;

use std::time::Duration;

use anyhow::Result;
use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use clap::{Parser, Subcommand};
use config::Config;
use state::AppState;
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
            let token = auth::issue_api_token(&state, user.id).await?;
            println!("{token}");
            Ok(())
        }
    }
}

async fn serve(state: AppState, bind: std::net::SocketAddr) -> Result<()> {
    let router = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .merge(auth::router())
        .merge(plans::router())
        .merge(web::router())
        .layer(CatchPanicLayer::new())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(state);
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

async fn index() -> &'static str {
    "Brume publishes agent plans. Use the Brume CLI to deploy one."
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.database)
        .await
    {
        Ok(1) => (StatusCode::OK, "ok"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, "database unavailable"),
    }
}
