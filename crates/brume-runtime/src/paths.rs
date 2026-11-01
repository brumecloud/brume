//! Where each service finds its sockets and files: under the Brume home with `brume dev`, at
//! the FHS paths on a host.

use std::path::{Path, PathBuf};

/// Where the data of a host lives: one directory per service, owned by its user.
const SYSTEM_DATA_DIR: &str = "/var/lib/brume";

/// Where the sockets of a host live.
const SYSTEM_RUN_DIR: &str = "/run/brume";

/// The layout of a Brume home, or of a host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    /// The databases, the master key and the bucket.
    home: PathBuf,
    /// The sockets.
    run: PathBuf,
}

impl Paths {
    /// The layout of `brume dev` under `home`: the sockets in `<home>/run`.
    pub fn new(home: PathBuf) -> Self {
        Self {
            run: home.join("run"),
            home,
        }
    }

    /// The layout of a host: the data in `/var/lib/brume`, the sockets in `/run/brume`.
    ///
    /// `brume install` creates the directories: `engine`, `telemetry` and `bucket`, owned by
    /// their service's user, and the run directory, whose group the four users of the services
    /// share, so that each socket's group lets the other services connect.
    pub fn system() -> Self {
        Self {
            home: PathBuf::from(SYSTEM_DATA_DIR),
            run: PathBuf::from(SYSTEM_RUN_DIR),
        }
    }

    /// The data directory: the Brume home, or `/var/lib/brume` on a host.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The directory of the sockets.
    pub fn run_dir(&self) -> PathBuf {
        self.run.clone()
    }

    /// `brume api` to the engine.
    pub fn api_socket(&self) -> PathBuf {
        self.run.join("api.sock")
    }

    /// Local commands, such as `brume dev`, to the engine.
    pub fn admin_socket(&self) -> PathBuf {
        self.run.join("admin.sock")
    }

    /// The agent to the engine.
    pub fn agent_socket(&self) -> PathBuf {
        self.run.join("agent.sock")
    }

    /// The agent's container logs to telemetry.
    pub fn telemetry_ingest_socket(&self) -> PathBuf {
        self.run.join("telemetry-ingest.sock")
    }

    /// The engine's log queries to telemetry.
    pub fn telemetry_query_socket(&self) -> PathBuf {
        self.run.join("telemetry-query.sock")
    }

    /// The engine's SQLite database.
    pub fn engine_db(&self) -> PathBuf {
        self.home.join("engine").join("state.sqlite")
    }

    /// The key that seals the secrets, readable by its owner only.
    pub fn master_key(&self) -> PathBuf {
        self.home.join("engine").join("master.key")
    }

    /// Telemetry's DuckDB database.
    pub fn telemetry_db(&self) -> PathBuf {
        self.home.join("telemetry").join("logs.duckdb")
    }

    /// The local bucket, where the engine ships its log.
    pub fn bucket_dir(&self) -> PathBuf {
        self.home.join("bucket")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Paths;

    #[test]
    fn layout_is_under_the_home() {
        let paths = Paths::new(PathBuf::from("/b"));
        assert_eq!(paths.run_dir(), PathBuf::from("/b/run"));
        assert_eq!(paths.api_socket(), PathBuf::from("/b/run/api.sock"));
        assert_eq!(paths.admin_socket(), PathBuf::from("/b/run/admin.sock"));
        assert_eq!(paths.agent_socket(), PathBuf::from("/b/run/agent.sock"));
        assert_eq!(
            paths.telemetry_ingest_socket(),
            PathBuf::from("/b/run/telemetry-ingest.sock")
        );
        assert_eq!(
            paths.telemetry_query_socket(),
            PathBuf::from("/b/run/telemetry-query.sock")
        );
        assert_eq!(paths.engine_db(), PathBuf::from("/b/engine/state.sqlite"));
        assert_eq!(paths.master_key(), PathBuf::from("/b/engine/master.key"));
        assert_eq!(
            paths.telemetry_db(),
            PathBuf::from("/b/telemetry/logs.duckdb")
        );
        assert_eq!(paths.bucket_dir(), PathBuf::from("/b/bucket"));
    }

    #[test]
    fn paths_system_layout() {
        let paths = Paths::system();
        assert_eq!(paths.home(), PathBuf::from("/var/lib/brume"));
        assert_eq!(paths.run_dir(), PathBuf::from("/run/brume"));
        assert_eq!(paths.api_socket(), PathBuf::from("/run/brume/api.sock"));
        assert_eq!(paths.admin_socket(), PathBuf::from("/run/brume/admin.sock"));
        assert_eq!(paths.agent_socket(), PathBuf::from("/run/brume/agent.sock"));
        assert_eq!(
            paths.telemetry_ingest_socket(),
            PathBuf::from("/run/brume/telemetry-ingest.sock")
        );
        assert_eq!(
            paths.telemetry_query_socket(),
            PathBuf::from("/run/brume/telemetry-query.sock")
        );
        assert_eq!(
            paths.engine_db(),
            PathBuf::from("/var/lib/brume/engine/state.sqlite")
        );
        assert_eq!(
            paths.master_key(),
            PathBuf::from("/var/lib/brume/engine/master.key")
        );
        assert_eq!(
            paths.telemetry_db(),
            PathBuf::from("/var/lib/brume/telemetry/logs.duckdb")
        );
        assert_eq!(paths.bucket_dir(), PathBuf::from("/var/lib/brume/bucket"));
    }
}
