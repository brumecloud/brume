//! `brume dev`: runs telemetry, the engine, the agent and the API as child processes of this
//! binary, prints a login link, and stops them all on Ctrl-C or as soon as one of them stops.

use std::io::{BufRead as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::os::unix::fs::DirBuilderExt as _;
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use anyhow::{Context as _, anyhow, bail};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_runtime::framed::{FramedReader, FramedWriter};
use brume_runtime::paths::Paths;
use clap::Args;
use tokio::net::{TcpStream, UnixStream};
use tokio::process::{Child, Command};
use tokio::signal::unix::{SignalKind, signal};

use crate::DEFAULT_PORT;

/// The longest wait for a service's socket, and for the login link.
const READY_TIMEOUT: Duration = Duration::from_secs(10);

/// How often a socket is tried while a service starts.
const READY_POLL: Duration = Duration::from_millis(50);

/// What `brume dev` says when the engine closes `admin.sock` without a link, or never answers.
pub const LOGIN_LINK_REFUSED: &str =
    "The engine refused to create a login link. Check its log above.";

#[derive(Debug, Args)]
pub struct DevArgs {
    /// The Brume home of this machine's services.
    #[arg(long, default_value = "./.brume-dev")]
    home: PathBuf,
    /// The port of the API and the panel, on 127.0.0.1.
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
    /// Deletes the home first, after confirmation: the event log, the databases, the bucket and
    /// the master key.
    #[arg(long)]
    reset: bool,
}

/// A service, in start order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Service {
    Telemetry,
    Engine,
    Agent,
    Api,
}

impl Service {
    const START_ORDER: [Self; 4] = [Self::Telemetry, Self::Engine, Self::Agent, Self::Api];

    fn name(self) -> &'static str {
        match self {
            Self::Telemetry => "telemetry",
            Self::Engine => "engine",
            Self::Agent => "agent",
            Self::Api => "api",
        }
    }
}

/// A running service.
struct Running {
    service: Service,
    child: Child,
    pid: u32,
}

/// Why the services stop.
enum Stop {
    /// Ctrl-C or SIGTERM.
    Asked,
    /// A service stopped by itself.
    Died(Service, ExitStatus),
    /// A service could not start, or the engine gave no login link.
    Failed(anyhow::Error),
}

/// Runs every service until Ctrl-C (success) or until one of them stops (an error).
pub async fn run(args: DevArgs) -> anyhow::Result<()> {
    // Before a reset: a port in use may be another `brume dev` on the same home.
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, args.port)))
        .with_context(|| {
            format!(
                "Port {} on 127.0.0.1 is in use. Stop what listens there, or pass --port.",
                args.port
            )
        })?;
    drop(listener);
    if args.reset && !reset(&args.home)? {
        bail!("Nothing was deleted.");
    }
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&args.home)
        .with_context(|| format!("Cannot create {}.", args.home.display()))?;
    let home = std::fs::canonicalize(&args.home)
        .with_context(|| format!("Cannot resolve {}.", args.home.display()))?;
    let paths = Paths::new(home);
    let mut interrupt = signal(SignalKind::interrupt()).context("Cannot handle Ctrl-C.")?;
    let mut terminate = signal(SignalKind::terminate()).context("Cannot handle SIGTERM.")?;

    let mut running: Vec<Running> = Vec::new();
    let started = tokio::select! {
        started = start_all(&paths, args.port, &mut running) => started,
        _ = interrupt.recv() => Err(Stop::Asked),
        _ = terminate.recv() => Err(Stop::Asked),
    };
    let stop = match started {
        Ok(token) => {
            print_ready(&paths, args.port, &token, &running);
            tokio::select! {
                died = first_exit(&mut running) => died,
                _ = interrupt.recv() => Stop::Asked,
                _ = terminate.recv() => Stop::Asked,
            }
        }
        Err(stop) => stop,
    };
    stop_all(&mut running).await;
    match stop {
        Stop::Asked => {
            eprintln!("Stopped.");
            Ok(())
        }
        Stop::Died(service, status) => Err(died(service, status)),
        Stop::Failed(error) => Err(error),
    }
}

/// Asks before deleting `home`. Whether it is gone.
fn reset(home: &Path) -> anyhow::Result<bool> {
    if !home.exists() {
        return Ok(true);
    }
    print!(
        "Delete {} and everything in it (event log, databases, bucket, master key)? [y/N] ",
        home.display()
    );
    std::io::stdout()
        .flush()
        .context("Cannot write the question.")?;
    let mut answer = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut answer)
        .context("Cannot read the answer.")?;
    if !matches!(answer.trim(), "y" | "Y" | "yes") {
        return Ok(false);
    }
    std::fs::remove_dir_all(home).with_context(|| format!("Cannot delete {}.", home.display()))?;
    println!("Deleted {}.", home.display());
    Ok(true)
}

/// Starts each service in order and waits for it, then asks the engine for a login link.
/// Running services are pushed to `running`, so that they stop whatever happens.
async fn start_all(paths: &Paths, port: u16, running: &mut Vec<Running>) -> Result<String, Stop> {
    let binary = std::env::current_exe()
        .context("Cannot find the brume binary.")
        .map_err(Stop::Failed)?;
    for service in Service::START_ORDER {
        let mut command = Command::new(&binary);
        command
            .arg(service.name())
            .arg("--home")
            .arg(paths.home())
            .stdin(Stdio::null())
            .kill_on_drop(true);
        if service == Service::Api {
            command.arg("--port").arg(port.to_string());
        }
        let child = command
            .spawn()
            .with_context(|| format!("Cannot start the {} service.", service.name()))
            .map_err(Stop::Failed)?;
        let pid = child.id().unwrap_or_default();
        running.push(Running {
            service,
            child,
            pid,
        });
        tokio::select! {
            ready = wait_ready(service, paths, port) => {
                if !ready {
                    return Err(Stop::Failed(anyhow!(
                        "The {} service did not get ready within {} s. Check its log above.",
                        service.name(),
                        READY_TIMEOUT.as_secs()
                    )));
                }
            }
            died = first_exit(running) => return Err(died),
        }
    }
    let admin_socket = paths.admin_socket();
    tokio::select! {
        token = login_token(&admin_socket, READY_TIMEOUT) => token.map_err(Stop::Failed),
        died = first_exit(running) => Err(died),
    }
}

/// Whether `service` listens within `READY_TIMEOUT`. A socket file is not enough: it may be
/// left by an earlier run, so each one must accept a connection. The agent listens on nothing.
async fn wait_ready(service: Service, paths: &Paths, port: u16) -> bool {
    let probe = async {
        loop {
            let ready = match service {
                Service::Telemetry => {
                    connects(&paths.telemetry_ingest_socket()).await
                        && connects(&paths.telemetry_query_socket()).await
                }
                // The engine binds `api.sock`, `admin.sock`, then `agent.sock`. A connection to
                // `agent.sock` would count as an agent, and the agent retries anyway.
                Service::Engine => {
                    connects(&paths.api_socket()).await && connects(&paths.admin_socket()).await
                }
                Service::Agent => true,
                Service::Api => TcpStream::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
                    .await
                    .is_ok(),
            };
            if ready {
                return;
            }
            tokio::time::sleep(READY_POLL).await;
        }
    };
    tokio::time::timeout(READY_TIMEOUT, probe).await.is_ok()
}

async fn connects(socket: &Path) -> bool {
    UnixStream::connect(socket).await.is_ok()
}

/// Asks the engine for a one-time login link on `admin.sock`, for `timeout` at most. A
/// read-only or failing engine closes the connection without an answer.
async fn login_token(admin_socket: &Path, timeout: Duration) -> anyhow::Result<String> {
    let ask = async {
        let stream = UnixStream::connect(admin_socket)
            .await
            .context("cannot connect to admin.sock")?;
        let (read, write) = stream.into_split();
        let mut writer = FramedWriter::new(write);
        writer
            .write(&AdminRequest::CreateLoginLink)
            .await
            .context("cannot send the request")?;
        let mut reader = FramedReader::new(read);
        reader
            .read::<AdminReply>()
            .await
            .context("cannot read the answer")?
            .context("the engine closed admin.sock without an answer")
    };
    let cause = match tokio::time::timeout(timeout, ask).await {
        Ok(Ok(AdminReply::LoginLink { token, .. })) => return Ok(token),
        Ok(Err(cause)) => format!("{cause:#}"),
        Err(_) => format!("no answer within {} ms", timeout.as_millis()),
    };
    tracing::warn!(%cause, "no login link");
    Err(anyhow!(LOGIN_LINK_REFUSED))
}

/// The login link, the services and their pids.
fn print_ready(paths: &Paths, port: u16, token: &str, running: &[Running]) {
    println!();
    println!("Brume runs from {}.", paths.home().display());
    println!("Open http://127.0.0.1:{port}/login#token={token}");
    println!();
    for running in running {
        println!("  {:<9}  pid {}", running.service.name(), running.pid);
    }
    println!();
    println!("Press Ctrl-C to stop.");
    let _ = std::io::stdout().flush();
}

/// The first service to stop, and how. Pending while none does; forever if there is none.
async fn first_exit(running: &mut [Running]) -> Stop {
    if running.is_empty() {
        return std::future::pending().await;
    }
    let waits = running.iter_mut().map(|running| {
        let service = running.service;
        Box::pin(async move { (service, running.child.wait().await) })
    });
    let ((service, status), _, _) = futures_util::future::select_all(waits).await;
    match status {
        // A terminal's Ctrl-C reaches every process of the group, the services too.
        Ok(status) if status.signal() == Some(SIGINT) => Stop::Asked,
        Ok(status) => Stop::Died(service, status),
        Err(error) => Stop::Failed(
            anyhow::Error::new(error)
                .context(format!("Cannot watch the {} service.", service.name())),
        ),
    }
}

/// SIGINT, which a terminal sends to the whole foreground group on Ctrl-C.
const SIGINT: i32 = 2;

/// Stops the services, the last started first, and waits for each one. They keep no state that
/// a kill can corrupt: SQLite and DuckDB recover from their journals.
async fn stop_all(running: &mut Vec<Running>) {
    while let Some(mut running) = running.pop() {
        if let Ok(None) = running.child.try_wait() {
            let _ = running.child.kill().await;
        }
    }
}

/// The error for a service that stopped by itself.
fn died(service: Service, status: ExitStatus) -> anyhow::Error {
    match status.code() {
        Some(code) => anyhow!(
            "The {} service stopped with code {code}. All services are stopped.",
            service.name()
        ),
        None => anyhow!(
            "The {} service stopped ({status}). All services are stopped.",
            service.name()
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt as _;
    use std::process::ExitStatus;
    use std::time::Duration;

    use super::{LOGIN_LINK_REFUSED, Service, died, login_token};

    #[tokio::test]
    async fn an_engine_that_closes_admin_sock_refuses_the_link() {
        let dir = tempfile::Builder::new()
            .prefix("brume-dev")
            .tempdir_in("/tmp")
            .expect("a temporary directory");
        let socket = dir.path().join("admin.sock");
        let listener = tokio::net::UnixListener::bind(&socket).expect("binds");
        tokio::spawn(async move {
            // Reads the request, then closes without an answer, as a read-only engine does.
            let (stream, _) = listener.accept().await.expect("accepts");
            let mut reader = brume_runtime::framed::FramedReader::new(stream);
            let _ = reader
                .read::<brume_proto::admin::AdminRequest>()
                .await
                .expect("a request");
        });
        let error = login_token(&socket, Duration::from_secs(10))
            .await
            .expect_err("refused");
        assert_eq!(error.to_string(), LOGIN_LINK_REFUSED);
    }

    #[tokio::test]
    async fn an_engine_that_never_answers_refuses_the_link_after_the_timeout() {
        let dir = tempfile::Builder::new()
            .prefix("brume-dev")
            .tempdir_in("/tmp")
            .expect("a temporary directory");
        let socket = dir.path().join("admin.sock");
        let listener = tokio::net::UnixListener::bind(&socket).expect("binds");
        let silent = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accepts");
            std::future::pending::<()>().await;
            drop(stream);
        });
        let error = login_token(&socket, Duration::from_millis(200))
            .await
            .expect_err("refused");
        assert_eq!(error.to_string(), LOGIN_LINK_REFUSED);
        silent.abort();
    }

    #[tokio::test]
    async fn an_engine_without_a_socket_refuses_the_link() {
        let error = login_token(
            std::path::Path::new("/nonexistent/admin.sock"),
            Duration::from_secs(10),
        )
        .await
        .expect_err("refused");
        assert_eq!(error.to_string(), LOGIN_LINK_REFUSED);
    }

    #[test]
    fn a_dead_service_is_named_with_its_code() {
        let message = died(Service::Agent, ExitStatus::from_raw(3 << 8)).to_string();
        assert_eq!(
            message,
            "The agent service stopped with code 3. All services are stopped."
        );
        let killed = died(Service::Engine, ExitStatus::from_raw(9)).to_string();
        assert!(
            killed.starts_with("The engine service stopped (signal: 9"),
            "{killed}"
        );
    }
}
