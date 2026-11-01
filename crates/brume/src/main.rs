//! The `brume` binary: runs one service (`engine`, `agent`, `api`, `telemetry`), under a Brume
//! home or on a host with `--system`, or all of them on this machine with `brume dev`. On a
//! host, `brume login` prints a login link to the panel.

#![forbid(unsafe_code)]

mod admin;
mod dev;
mod logging;
mod login;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context as _;
use brume_api::{ApiConfig, ApiFileConfig, DEFAULT_PORT, SYSTEM_CONFIG_PATH};
use brume_runtime::paths::Paths;
use brume_runtime::peer::Peers;
use clap::{Args, Parser, Subcommand};

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
    /// Prints a one-time login link to the panel. Run it as root on the host.
    Login,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct ServiceArgs {
    /// The Brume home: sockets, databases and the bucket live under it.
    #[arg(long)]
    home: Option<PathBuf>,
    /// Runs on a host, as the service's own system user: data in /var/lib/brume, sockets in
    /// /run/brume, each socket open to the users of its clients only.
    #[arg(long)]
    system: bool,
}

impl ServiceArgs {
    fn paths(&self) -> Paths {
        match &self.home {
            Some(home) => Paths::new(home.clone()),
            None => Paths::system(),
        }
    }

    /// The users that the service accepts on its sockets: the system users of a host, resolved
    /// once at startup, or the owner of the home's run directory.
    fn peers(&self, paths: &Paths) -> anyhow::Result<Peers> {
        if self.system {
            Peers::system().context("cannot resolve the users of the services")
        } else {
            let run_dir = paths.run_dir();
            Peers::same_user(&run_dir)
                .with_context(|| format!("cannot create {}", run_dir.display()))
        }
    }
}

#[derive(Debug, Args)]
struct ApiArgs {
    #[command(flatten)]
    service: ServiceArgs,
    /// The port on 127.0.0.1. With --system, /etc/brume/api.toml sets it.
    #[arg(long, default_value_t = DEFAULT_PORT, conflicts_with = "system")]
    port: u16,
}

impl ApiArgs {
    fn config(&self) -> anyhow::Result<ApiConfig> {
        if self.service.system {
            let file = ApiFileConfig::load(Path::new(SYSTEM_CONFIG_PATH))?;
            Ok(ApiConfig::from_file(&file))
        } else {
            Ok(ApiConfig::new(self.port))
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let name = match &cli.command {
        Command::Engine(_) => "engine",
        Command::Agent(_) => "agent",
        Command::Api(_) => "api",
        Command::Telemetry(_) => "telemetry",
        Command::Dev(_) => "dev",
        Command::Login => "login",
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
            Command::Engine(args) => {
                let paths = args.paths();
                let peers = args.peers(&paths)?;
                brume_engine::run(paths, peers).await
            }
            Command::Agent(args) => brume_agent::run(args.paths()).await,
            Command::Api(args) => {
                let config = args.config()?;
                brume_api::run(args.service.paths(), config).await
            }
            Command::Telemetry(args) => {
                let paths = args.paths();
                let peers = args.peers(&paths)?;
                brume_telemetry::run(paths, peers).await
            }
            Command::Dev(args) => dev::run(args).await,
            Command::Login => login::run(),
        }
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        // `brume dev` and `brume login` speak to a person: the message is the whole error.
        Err(error) if matches!(name, "dev" | "login") => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
        Err(error) => {
            tracing::error!(error = format!("{error:#}"), "brume {name} stopped");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use brume_runtime::paths::Paths;
    use clap::Parser as _;

    use super::{Cli, Command};

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("brume").chain(args.iter().copied()))
    }

    #[test]
    fn a_service_runs_under_a_home_or_on_the_host() {
        let Ok(Cli {
            command: Command::Engine(args),
        }) = parse(&["engine", "--system"])
        else {
            panic!("parses");
        };
        assert_eq!(args.paths(), Paths::system());
        let Ok(Cli {
            command: Command::Telemetry(args),
        }) = parse(&["telemetry", "--home", "/b"])
        else {
            panic!("parses");
        };
        assert_eq!(args.paths(), Paths::new(PathBuf::from("/b")));
        for service in ["engine", "agent", "api", "telemetry"] {
            assert!(parse(&[service]).is_err(), "{service} needs a layout");
            assert!(
                parse(&[service, "--home", "/b", "--system"]).is_err(),
                "{service} takes one layout"
            );
        }
    }

    #[test]
    fn the_port_of_a_host_is_in_its_api_toml() {
        assert!(parse(&["api", "--system"]).is_ok());
        assert!(parse(&["api", "--system", "--port", "8080"]).is_err());
        let Ok(Cli {
            command: Command::Api(args),
        }) = parse(&["api", "--home", "/b", "--port", "8080"])
        else {
            panic!("parses");
        };
        assert_eq!(args.port, 8080);
        assert!(matches!(
            parse(&["login"]),
            Ok(Cli {
                command: Command::Login
            })
        ));
    }
}
