//! The telemetry service. It stores container logs in a hardened DuckDB and serves them,
//! history then live, to the engine.

#![forbid(unsafe_code)]

mod ingest;
pub mod level;
mod query;
pub mod store;

use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use brume_proto::telemetry::RawLogLine;
use brume_runtime::paths::Paths;
use brume_runtime::peer::{Admission, Peers, SocketRole, peer_uid};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;

use crate::store::{Reader, Store, StoredLine};

/// Lines kept for live queries that read slower than the agent writes, at most 64 MiB of
/// messages. A query that falls further behind catches up from the database.
const LIVE_CAPACITY: usize = 4_096;

/// Lines older than this are deleted.
const RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// How often the old lines are deleted.
const RETENTION_PERIOD: Duration = Duration::from_secs(60 * 60);

/// Runs the service until a socket fails: stores the batches of `telemetry-ingest.sock` and
/// answers the queries of `telemetry-query.sock`, for `peers` only.
pub async fn run(paths: Paths, peers: Peers) -> anyhow::Result<()> {
    let db = paths.telemetry_db();
    if let Some(dir) = db.parent() {
        create_private_dir(dir)?;
    }
    create_private_dir(&paths.telemetry_run_dir())?;
    let store = Store::open(&db)?;
    let shared = Arc::new(Shared::new(store)?);
    let ingest = listen(&peers, &paths.telemetry_ingest_socket())?;
    let query = listen(&peers, &paths.telemetry_query_socket())?;
    tracing::info!(run_dir = %paths.telemetry_run_dir().display(), "telemetry is listening");
    tokio::try_join!(
        ingest::serve(ingest, peers, Arc::clone(&shared)),
        query::serve(query, peers, Arc::clone(&shared)),
        retain(Arc::clone(&shared)),
    )?;
    Ok(())
}

/// The state that the connections share.
pub(crate) struct Shared {
    store: Mutex<Store>,
    /// The connection that queries clone theirs from. Its own lock, which inserts and deletes
    /// never take, so a query never waits for them.
    readers: Mutex<Reader>,
    live: broadcast::Sender<StoredLine>,
}

impl Shared {
    fn new(store: Store) -> anyhow::Result<Self> {
        let (live, _) = broadcast::channel(LIVE_CAPACITY);
        Ok(Self {
            readers: Mutex::new(store.reader()?),
            store: Mutex::new(store),
            live,
        })
    }

    /// Stores `lines` and publishes them to the live queries. Blocking. Publishing under the
    /// lock keeps the ids in order on the channel.
    pub(crate) fn insert(&self, lines: Vec<RawLogLine>) -> anyhow::Result<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("the log store lock is poisoned"))?;
        for line in store.insert(lines)? {
            // An error only means that no query follows.
            let _ = self.live.send(line);
        }
        Ok(())
    }

    /// The new lines, from now on.
    pub(crate) fn subscribe(&self) -> broadcast::Receiver<StoredLine> {
        self.live.subscribe()
    }

    /// A connection of its own for a query.
    pub(crate) fn reader(&self) -> anyhow::Result<Reader> {
        self.readers
            .lock()
            .map_err(|_| anyhow::anyhow!("the log reader lock is poisoned"))?
            .try_clone()
    }

    /// Deletes the lines older than `RETENTION`. Blocking.
    fn delete_old_lines(&self) -> anyhow::Result<usize> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("the clock is before 1970")?
            .as_millis();
        let cutoff_ms = u64::try_from(now_ms.saturating_sub(RETENTION.as_millis())).unwrap_or(0);
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("the log store lock is poisoned"))?
            .delete_before(cutoff_ms)
    }
}

/// Deletes old lines now, then every `RETENTION_PERIOD`. A failure is logged and retried at the
/// next period.
async fn retain(shared: Arc<Shared>) -> anyhow::Result<()> {
    let mut ticks = tokio::time::interval(RETENTION_PERIOD);
    loop {
        ticks.tick().await;
        let shared = Arc::clone(&shared);
        match tokio::task::spawn_blocking(move || shared.delete_old_lines()).await {
            Ok(Ok(deleted)) => tracing::debug!(deleted, "old log lines deleted"),
            Ok(Err(error)) => {
                tracing::warn!(error = format!("{error:#}"), "cannot delete old log lines");
            }
            Err(error) => tracing::warn!(%error, "the log retention task panicked"),
        }
    }
}

/// Creates `dir` and its parents, readable by the owner only when it creates them.
fn create_private_dir(dir: &Path) -> anyhow::Result<()> {
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .with_context(|| format!("cannot create {}", dir.display()))
}

/// Listens on `path` for `peers`. A socket file left by an earlier run is replaced.
fn listen(peers: &Peers, path: &Path) -> anyhow::Result<UnixListener> {
    peers
        .listen(path)
        .with_context(|| format!("cannot listen on {}", path.display()))
}

/// Whether the peer of a new connection on `socket` may use it. Logs a refusal.
pub(crate) fn admit(peers: &Peers, socket: SocketRole, stream: &UnixStream) -> bool {
    match peers.admit(socket, peer_uid(stream)) {
        Admission::Allowed => true,
        Admission::Refused { uid } => {
            tracing::warn!(?socket, uid, "refused a connection from another user");
            false
        }
        Admission::Gone => false,
    }
}
