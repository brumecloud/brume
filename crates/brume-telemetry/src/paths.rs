//! The files of telemetry under the Brume home.
//!
//! A stand-in for `brume_runtime::paths::Paths`, with the same names and layout, until that
//! crate lands: switch to it then and delete this module.

use std::path::PathBuf;

/// The paths under a Brume home.
#[derive(Clone, Debug)]
pub struct Paths {
    home: PathBuf,
}

impl Paths {
    /// The paths under `home`.
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    /// `<home>/run`, which holds the sockets.
    pub fn run_dir(&self) -> PathBuf {
        self.home.join("run")
    }

    /// Where the agent sends `LogBatch`es.
    pub fn telemetry_ingest_socket(&self) -> PathBuf {
        self.run_dir().join("telemetry-ingest.sock")
    }

    /// Where the engine sends `LogQuery`s.
    pub fn telemetry_query_socket(&self) -> PathBuf {
        self.run_dir().join("telemetry-query.sock")
    }

    /// The DuckDB file of the logs.
    pub fn telemetry_db(&self) -> PathBuf {
        self.home.join("telemetry").join("logs.duckdb")
    }
}
