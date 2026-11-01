//! Where each service finds its sockets and files, under the Brume home.

use std::path::{Path, PathBuf};

/// The layout of a Brume home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    home: PathBuf,
}

impl Paths {
    /// The layout under `home`.
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    /// The Brume home.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The directory of the sockets: `<home>/run`. Its owner is the only user the services
    /// talk to.
    pub fn run_dir(&self) -> PathBuf {
        self.home.join("run")
    }

    /// `brume api` to the engine.
    pub fn api_socket(&self) -> PathBuf {
        self.run_dir().join("api.sock")
    }

    /// Local commands, such as `brume dev`, to the engine.
    pub fn admin_socket(&self) -> PathBuf {
        self.run_dir().join("admin.sock")
    }

    /// The agent to the engine.
    pub fn agent_socket(&self) -> PathBuf {
        self.run_dir().join("agent.sock")
    }

    /// The agent's container logs to telemetry.
    pub fn telemetry_ingest_socket(&self) -> PathBuf {
        self.run_dir().join("telemetry-ingest.sock")
    }

    /// The engine's log queries to telemetry.
    pub fn telemetry_query_socket(&self) -> PathBuf {
        self.run_dir().join("telemetry-query.sock")
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
}
