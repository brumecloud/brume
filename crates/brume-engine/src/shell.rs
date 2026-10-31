//! The engine service: one loop owns the `Engine`; connection tasks feed it inputs and get its
//! outputs back through channels.

use std::collections::BTreeMap;
use std::fs::DirBuilder;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use brume_domain::{Ctx, Timestamp};
use brume_engine_core::{ClientId, Engine, Input, Output};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_proto::agent::{AgentToEngine, EngineToAgent};
use brume_proto::api::{ApiReply, ApiRequest};
use brume_proto::telemetry::LogQuery;
use brume_runtime::crypto::MasterKey;
use brume_runtime::framed::{FramedReader, FramedWriter};
use brume_runtime::object_store::DirStore;
use brume_runtime::paths::Paths;
use brume_runtime::peer::{owner_uid, peer_uid};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio::time::Instant;

use crate::relay::relay;
use crate::shipper::{SHIP_PERIOD, ShipOutcome, Shipper};
use crate::sqlite::SqliteStorage;

/// How many messages wait for the loop before senders slow down.
const LOOP_QUEUE: usize = 1024;

/// The shell's clock and randomness: the system's.
struct SystemCtx;

impl Ctx for SystemCtx {
    fn now(&self) -> Timestamp {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        Timestamp::from_millis(u64::try_from(since_epoch.as_millis()).unwrap_or(u64::MAX))
    }

    fn fill_random(&mut self, buf: &mut [u8]) {
        rand::fill(buf);
    }
}

/// What reaches the loop.
enum LoopMessage {
    Input(Input),
    /// A connection of `brume api` or of a local command, before its first input.
    Client {
        client: ClientId,
        sender: mpsc::UnboundedSender<ClientMessage>,
    },
    ClientGone(ClientId),
    Agent {
        connection: u64,
        sender: mpsc::UnboundedSender<EngineToAgent>,
    },
    AgentGone {
        connection: u64,
    },
    /// Another server ships to the bucket.
    ReadOnly,
}

/// What the loop sends to a client connection.
enum ClientMessage {
    Reply(ApiReply),
    Admin(AdminReply),
    /// Hand the connection over to a log stream.
    Stream {
        request: u64,
        query: LogQuery,
    },
}

/// Which socket a connection came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Socket {
    Api,
    Admin,
    Agent,
}

/// How long to wait before accepting again after a failed accept, such as too many open files.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(100);

/// Runs the engine: opens the database and the master key, listens on `api.sock`,
/// `admin.sock` and `agent.sock`, and ships the log to the bucket. Returns on a fatal error.
pub async fn run(paths: Paths) -> anyhow::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(paths.run_dir())
        .context("creating the run directory")?;
    let owner = owner_uid(&paths.run_dir()).context("reading the run directory's owner")?;
    let key = MasterKey::load_or_create(&paths.master_key()).context("loading the master key")?;
    let storage = SqliteStorage::open(&paths.engine_db(), key).context("opening the database")?;
    let mut ctx = SystemCtx;
    let mut engine = Engine::open(storage, &mut ctx).context("replaying the event log")?;
    tracing::info!(position = engine.position(), "the event log is replayed");

    let (sender, receiver) = mpsc::channel(LOOP_QUEUE);
    for (socket, path) in [
        (Socket::Api, paths.api_socket()),
        (Socket::Admin, paths.admin_socket()),
        (Socket::Agent, paths.agent_socket()),
    ] {
        let listener = bind(&path)?;
        tokio::spawn(accept(
            listener,
            socket,
            owner,
            sender.clone(),
            paths.telemetry_query_socket(),
        ));
    }
    let shipper = Shipper::open(&paths.engine_db(), DirStore::new(paths.bucket_dir()))
        .context("opening the shipper")?;
    tokio::spawn(ship(shipper, sender.clone()));
    drop(sender);

    let mut out = Vec::new();
    engine.handle(&mut ctx, Input::Tick, &mut out);
    serve(engine, ctx, receiver, out).await
}

/// Binds a socket readable by its owner only, replacing a stale one.
fn bind(path: &Path) -> anyhow::Result<UnixListener> {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("removing a stale socket"),
    }
    let listener =
        UnixListener::bind(path).with_context(|| format!("binding {}", path.display()))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .context("restricting a socket to its owner")?;
    Ok(listener)
}

/// Whether a peer may talk to the engine: only the user that owns the run directory.
fn peer_allowed(peer: std::io::Result<u32>, owner: u32) -> bool {
    matches!(peer, Ok(uid) if uid == owner)
}

async fn accept(
    listener: UnixListener,
    socket: Socket,
    owner: u32,
    sender: mpsc::Sender<LoopMessage>,
    telemetry: PathBuf,
) {
    let mut next_connection = 0u64;
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(error) => {
                tracing::warn!(%error, ?socket, "accepting a connection failed");
                tokio::time::sleep(ACCEPT_BACKOFF).await;
                continue;
            }
        };
        if !peer_allowed(peer_uid(&stream), owner) {
            tracing::warn!(?socket, "refused a connection from another user");
            continue;
        }
        next_connection += 1;
        // Clients of all sockets share one id space: the socket is in the top bits.
        let id = next_connection
            | match socket {
                Socket::Api => 0,
                Socket::Admin => 1 << 62,
                Socket::Agent => 2 << 62,
            };
        let sender = sender.clone();
        let telemetry = telemetry.clone();
        tokio::spawn(async move {
            let result = match socket {
                Socket::Api => client(stream, false, ClientId(id), sender, &telemetry).await,
                Socket::Admin => client(stream, true, ClientId(id), sender, &telemetry).await,
                Socket::Agent => agent(stream, id, sender).await,
            };
            if let Err(error) = result {
                tracing::debug!(%error, ?socket, "a connection ended");
            }
        });
    }
}

/// A connection of `brume api` (multiplexed calls, or one log stream) or, with `admin`, of a
/// local command. However it ends, the loop forgets it.
async fn client(
    stream: UnixStream,
    admin: bool,
    client: ClientId,
    sender: mpsc::Sender<LoopMessage>,
    telemetry: &Path,
) -> std::io::Result<()> {
    let (read, write) = stream.into_split();
    let mut reader = FramedReader::new(read);
    let mut writer = FramedWriter::new(write);
    let (to_client, mut messages) = mpsc::unbounded_channel();
    let closed = || std::io::Error::from(std::io::ErrorKind::BrokenPipe);
    sender
        .send(LoopMessage::Client {
            client,
            sender: to_client,
        })
        .await
        .map_err(|_| closed())?;
    let result = loop {
        tokio::select! {
            input = read_input(&mut reader, admin, client) => match input {
                Ok(Some(input)) => {
                    if sender.send(LoopMessage::Input(input)).await.is_err() {
                        break Err(closed());
                    }
                }
                Ok(None) => break Ok(()),
                Err(error) => break Err(error),
            },
            message = messages.recv() => {
                let written = match message {
                    Some(ClientMessage::Reply(reply)) => writer.write(&reply).await,
                    Some(ClientMessage::Admin(reply)) => writer.write(&reply).await,
                    Some(ClientMessage::Stream { request, query }) => {
                        break relay(telemetry, request, query, reader, writer).await;
                    }
                    None => break Ok(()),
                };
                if let Err(error) = written {
                    break Err(error);
                }
            }
        }
    };
    let _ = sender.send(LoopMessage::ClientGone(client)).await;
    result
}

async fn read_input<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut FramedReader<R>,
    admin: bool,
    client: ClientId,
) -> std::io::Result<Option<Input>> {
    Ok(if admin {
        reader
            .read::<AdminRequest>()
            .await?
            .map(|request| Input::Admin { client, request })
    } else {
        reader
            .read::<ApiRequest>()
            .await?
            .map(|request| Input::Api { client, request })
    })
}

/// The agent's connection: its reports in, the desired state and jobs out. However it ends,
/// the loop learns that the agent is gone.
async fn agent(
    stream: UnixStream,
    connection: u64,
    sender: mpsc::Sender<LoopMessage>,
) -> std::io::Result<()> {
    let (read, write) = stream.into_split();
    let mut reader = FramedReader::new(read);
    let mut writer = FramedWriter::new(write);
    let (to_agent, mut messages) = mpsc::unbounded_channel();
    let closed = || std::io::Error::from(std::io::ErrorKind::BrokenPipe);
    sender
        .send(LoopMessage::Agent {
            connection,
            sender: to_agent,
        })
        .await
        .map_err(|_| closed())?;
    let result = loop {
        tokio::select! {
            report = reader.read::<AgentToEngine>() => match report {
                Ok(Some(report)) => {
                    if sender.send(LoopMessage::Input(Input::FromAgent(report))).await.is_err() {
                        break Err(closed());
                    }
                }
                Ok(None) => break Ok(()),
                Err(error) => break Err(error),
            },
            message = messages.recv() => match message {
                Some(message) => {
                    if let Err(error) = writer.write(&message).await {
                        break Err(error);
                    }
                }
                None => break Ok(()),
            },
        }
    };
    let _ = sender.send(LoopMessage::AgentGone { connection }).await;
    result
}

/// Ships a segment per period at most, until another server's object appears.
async fn ship(mut shipper: Shipper<DirStore>, sender: mpsc::Sender<LoopMessage>) {
    let mut period = tokio::time::interval(SHIP_PERIOD);
    loop {
        period.tick().await;
        let round = tokio::task::spawn_blocking(move || {
            let outcome = shipper.ship_once();
            (shipper, outcome)
        })
        .await;
        let outcome;
        (shipper, outcome) = match round {
            Ok(round) => round,
            Err(error) => {
                tracing::error!(%error, "the shipper stopped");
                return;
            }
        };
        match outcome {
            Ok(ShipOutcome::Idle) => {}
            Ok(ShipOutcome::Shipped { first_seq, count }) => {
                tracing::debug!(first_seq, count, "shipped a segment of the log");
            }
            Ok(ShipOutcome::Conflict { key }) => {
                tracing::error!(
                    %key,
                    "another server writes to this bucket: the engine is now read-only"
                );
                let _ = sender.send(LoopMessage::ReadOnly).await;
                return;
            }
            Err(error) => tracing::warn!(%error, "shipping the log failed; retrying"),
        }
    }
}

/// The loop: the only owner of the engine.
async fn serve(
    mut engine: Engine<SqliteStorage>,
    mut ctx: SystemCtx,
    mut receiver: mpsc::Receiver<LoopMessage>,
    mut out: Vec<Output>,
) -> anyhow::Result<()> {
    let mut clients: BTreeMap<ClientId, mpsc::UnboundedSender<ClientMessage>> = BTreeMap::new();
    let mut agent: Option<(u64, mpsc::UnboundedSender<EngineToAgent>)> = None;
    let mut wake: Option<Instant> = None;
    loop {
        for output in out.drain(..) {
            match output {
                Output::Reply { client, reply } => {
                    if let Some(sender) = clients.get(&client) {
                        let _ = sender.send(ClientMessage::Reply(reply));
                    }
                }
                Output::AdminReply { client, reply } => {
                    if let Some(sender) = clients.get(&client) {
                        let _ = sender.send(ClientMessage::Admin(reply));
                    }
                }
                Output::ToAgent(message) => {
                    if let Some((_, sender)) = &agent {
                        let _ = sender.send(message);
                    }
                }
                Output::OpenLogStream {
                    client,
                    request,
                    query,
                } => {
                    // The connection now belongs to the stream.
                    if let Some(sender) = clients.remove(&client) {
                        let _ = sender.send(ClientMessage::Stream { request, query });
                    }
                }
                Output::WakeAt(at) => {
                    let delay = at.as_millis().saturating_sub(ctx.now().as_millis());
                    wake = Some(Instant::now() + Duration::from_millis(delay));
                }
            }
        }

        let message = tokio::select! {
            message = receiver.recv() => message,
            () = sleep_until(wake) => {
                wake = None;
                Some(LoopMessage::Input(Input::Tick))
            }
        };
        let Some(message) = message else {
            anyhow::bail!("every listener stopped");
        };
        match message {
            LoopMessage::Input(input) => engine.handle(&mut ctx, input, &mut out),
            LoopMessage::Client { client, sender } => {
                clients.insert(client, sender);
            }
            LoopMessage::ClientGone(client) => {
                clients.remove(&client);
            }
            LoopMessage::Agent { connection, sender } => {
                agent = Some((connection, sender));
                if let Some(machine) = engine.machine().cloned() {
                    engine.handle(&mut ctx, Input::AgentConnected { machine }, &mut out);
                }
            }
            LoopMessage::AgentGone { connection } => {
                if agent
                    .as_ref()
                    .is_some_and(|(current, _)| *current == connection)
                {
                    agent = None;
                    engine.handle(&mut ctx, Input::AgentDisconnected, &mut out);
                }
            }
            LoopMessage::ReadOnly => engine.set_read_only(),
        }
    }
}

async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

#[cfg(test)]
mod tests {
    use super::peer_allowed;

    #[test]
    fn only_the_owner_of_the_run_directory_is_a_peer() {
        assert!(peer_allowed(Ok(501), 501));
        assert!(!peer_allowed(Ok(502), 501));
        assert!(!peer_allowed(Ok(0), 501));
        assert!(!peer_allowed(Err(std::io::ErrorKind::Other.into()), 501));
    }
}
