//! The `brume` binary: runs one service (`engine`, `agent`, `api`, `telemetry`), or all of them
//! on this machine with `brume dev`.

#![forbid(unsafe_code)]

mod dev;
mod logging;

use std::path::PathBuf;
use std::process::ExitCode;

use brume_runtime::paths::Paths;
use clap::{Args, Parser, Subcommand};

/// The port of the API and the panel, on 127.0.0.1.
const DEFAULT_PORT: u16 = 7878;

#[derive(Debug, Parser)]
#[command(name = "brume", version, about = "Brume, a PaaS on your own server.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Runs the engine: the event log, the sockets of the API, the admin and the agent.
    Engine(ServiceArgs),
    /// Runs the agent: drives Docker and forwards container logs.
    Agent(ServiceArgs),
    /// Runs the HTTP API and the panel on 127.0.0.1.
    Api(ApiArgs),
    /// Runs telemetry: stores and serves container logs.
    Telemetry(ServiceArgs),
    /// Runs every service on this machine, as child processes, until Ctrl-C.
    Dev(dev::DevArgs),
}

#[derive(Debug, Args)]
struct ServiceArgs {
    /// The Brume home: sockets, databases and the bucket live under it.
    #[arg(long)]
    home: PathBuf,
}

#[derive(Debug, Args)]
struct ApiArgs {
    #[command(flatten)]
    service: ServiceArgs,
    /// The port on 127.0.0.1.
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let name = match &cli.command {
        Command::Engine(_) => "engine",
        Command::Agent(_) => "agent",
        Command::Api(_) => "api",
        Command::Telemetry(_) => "telemetry",
        Command::Dev(_) => "dev",
    };
    logging::init(name);
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(%error, "cannot start the async runtime");
            return ExitCode::FAILURE;
        }
    };
    let result = runtime.block_on(async move {
        match cli.command {
            Command::Engine(args) => brume_engine::run(Paths::new(args.home)).await,
            Command::Agent(args) => brume_agent::run(Paths::new(args.home)).await,
            Command::Api(args) => {
                brume_api::run(
                    Paths::new(args.service.home),
                    brume_api::ApiConfig::new(args.port),
                )
                .await
            }
            Command::Telemetry(args) => brume_telemetry::run(Paths::new(args.home)).await,
            Command::Dev(args) => dev::run(args).await,
        }
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        // `brume dev` speaks to a person: its message is the whole error.
        Err(error) if name == "dev" => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
        Err(error) => {
            tracing::error!(error = format!("{error:#}"), "brume {name} stopped");
            ExitCode::FAILURE
        }
    }
}
