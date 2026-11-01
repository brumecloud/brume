//! Where each service finds its sockets and files: under the Brume home with `brume dev`, at
//! the FHS paths on a host.

use std::path::{Path, PathBuf};

/// Where the data of a host lives: one directory per service, owned by its user.
const SYSTEM_DATA_DIR: &str = "/var/lib/brume";

/// Where the sockets of a host live: one directory per listening service.
const SYSTEM_RUN_DIR: &str = "/run/brume";

/// The layout of a Brume home, or of a host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    /// The databases, the master key and the bucket.
    home: PathBuf,
    /// The parent of the socket directories.
    run: PathBuf,
    /// The sockets the engine listens on.
    engine_run: PathBuf,
    /// The sockets telemetry listens on.
    telemetry_run: PathBuf,
}

impl Paths {
    /// The layout of `brume dev` under `home`: the sockets in `<home>/run`.
    pub fn new(home: PathBuf) -> Self {
        let run = home.join("run");
        Self {
            engine_run: run.clone(),
            telemetry_run: run.clone(),
            run,
            home,
        }
    }

    /// The layout of a host: the data in `/var/lib/brume`, the sockets in `/run/brume/engine`
    /// and `/run/brume/telemetry`.
    ///
    /// `brume install` creates the directories: `engine`, `telemetry` and `bucket`, owned by
    /// their service's user, and the socket directories, each owned by the service that
    /// listens in it, setgid to the group the four users share and not writable by that group
    /// (`2750`). The other services can connect to a socket, but none can replace it.
    pub fn system() -> Self {
        let run = PathBuf::from(SYSTEM_RUN_DIR);
        Self {
            home: PathBuf::from(SYSTEM_DATA_DIR),
            engine_run: run.join("engine"),
            telemetry_run: run.join("telemetry"),
            run,
        }
    }

    /// The data directory: the Brume home, or `/var/lib/brume` on a host.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The parent of the socket directories. With `brume dev`, it holds every socket.
    pub fn run_dir(&self) -> PathBuf {
        self.run.clone()
    }

    /// The directory of the engine's sockets.
    pub fn engine_run_dir(&self) -> PathBuf {
        self.engine_run.clone()
    }

    /// The directory of telemetry's sockets.
    pub fn telemetry_run_dir(&self) -> PathBuf {
        self.telemetry_run.clone()
    }

    /// `brume api` to the engine.
    pub fn api_socket(&self) -> PathBuf {
        self.engine_run.join("api.sock")
    }

    /// Local commands, such as `brume dev`, to the engine.
    pub fn admin_socket(&self) -> PathBuf {
        self.engine_run.join("admin.sock")
    }

    /// The agent to the engine.
    pub fn agent_socket(&self) -> PathBuf {
        self.engine_run.join("agent.sock")
    }

    /// The agent's container logs to telemetry.
    pub fn telemetry_ingest_socket(&self) -> PathBuf {
        self.telemetry_run.join("telemetry-ingest.sock")
    }

    /// The engine's log queries to telemetry.
    pub fn telemetry_query_socket(&self) -> PathBuf {
        self.telemetry_run.join("telemetry-query.sock")
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

    /// The directory where the agent writes Traefik's config, which Traefik mounts read-only.
    /// `brume install` creates it on a host, owned by the agent's user; `brume dev` lets the
    /// agent create it.
    pub fn traefik_dir(&self) -> PathBuf {
        self.home.join("agent").join("traefik")
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
        assert_eq!(paths.traefik_dir(), PathBuf::from("/b/agent/traefik"));
    }

    #[test]
    fn paths_system_layout() {
        let paths = Paths::system();
        assert_eq!(paths.home(), PathBuf::from("/var/lib/brume"));
        assert_eq!(paths.run_dir(), PathBuf::from("/run/brume"));
        assert_eq!(paths.engine_run_dir(), PathBuf::from("/run/brume/engine"));
        assert_eq!(
            paths.telemetry_run_dir(),
            PathBuf::from("/run/brume/telemetry")
        );
        assert_eq!(
            paths.api_socket(),
            PathBuf::from("/run/brume/engine/api.sock")
        );
        assert_eq!(
            paths.admin_socket(),
            PathBuf::from("/run/brume/engine/admin.sock")
        );
        assert_eq!(
            paths.agent_socket(),
            PathBuf::from("/run/brume/engine/agent.sock")
        );
        assert_eq!(
            paths.telemetry_ingest_socket(),
            PathBuf::from("/run/brume/telemetry/telemetry-ingest.sock")
        );
        assert_eq!(
            paths.telemetry_query_socket(),
            PathBuf::from("/run/brume/telemetry/telemetry-query.sock")
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
        assert_eq!(
            paths.traefik_dir(),
            PathBuf::from("/var/lib/brume/agent/traefik")
        );
    }
}
