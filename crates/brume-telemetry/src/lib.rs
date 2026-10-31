//! The telemetry service. It stores container logs in a hardened DuckDB and serves them,
//! history then live, to the engine.

#![forbid(unsafe_code)]

pub mod framed;
mod ingest;
pub mod level;
pub mod paths;
mod query;
pub mod store;

use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use brume_proto::telemetry::RawLogLine;
use tokio::net::UnixListener;
use tokio::sync::broadcast;

use crate::paths::Paths;
use crate::store::{Reader, Store, StoredLine};

/// Lines kept for live queries that read slower than the agent writes. A query that falls
/// further behind catches up from the database.
const LIVE_CAPACITY: usize = 16_384;

/// Lines older than this are deleted.
const RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// How often the old lines are deleted.
const RETENTION_PERIOD: Duration = Duration::from_secs(60 * 60);

/// Runs the service under `home` until a socket fails: stores the batches of
/// `telemetry-ingest.sock` and answers the queries of `telemetry-query.sock`.
pub async fn run(home: PathBuf) -> anyhow::Result<()> {
    let paths = Paths::new(home);
    let db = paths.telemetry_db();
    if let Some(dir) = db.parent() {
        create_private_dir(dir)?;
    }
    create_private_dir(&paths.run_dir())?;
    let store = Store::open(&db)?;
    let shared = Arc::new(Shared::new(store));
    let ingest = bind(&paths.telemetry_ingest_socket())?;
    let query = bind(&paths.telemetry_query_socket())?;
    tracing::info!(home = %paths.run_dir().display(), "telemetry is listening");
    tokio::try_join!(
        ingest::serve(ingest, Arc::clone(&shared)),
        query::serve(query, Arc::clone(&shared)),
        retain(Arc::clone(&shared)),
    )?;
    Ok(())
}

/// The state that the connections share.
pub(crate) struct Shared {
    store: Mutex<Store>,
    live: broadcast::Sender<StoredLine>,
}

impl Shared {
    fn new(store: Store) -> Self {
        let (live, _) = broadcast::channel(LIVE_CAPACITY);
        Self {
            store: Mutex::new(store),
            live,
        }
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
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("the log store lock is poisoned"))?
            .reader()
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

/// Listens on `path`, owner only. A socket file left by an earlier run is replaced.
fn bind(path: &Path) -> anyhow::Result<UnixListener> {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("cannot remove {}", path.display()));
        }
    }
    let listener =
        UnixListener::bind(path).with_context(|| format!("cannot listen on {}", path.display()))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("cannot restrict {}", path.display()))?;
    Ok(listener)
}
