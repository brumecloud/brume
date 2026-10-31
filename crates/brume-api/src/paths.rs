//! The files of the API under the Brume home.
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

    /// The engine's socket for `brume api`.
    pub fn api_socket(&self) -> PathBuf {
        self.run_dir().join("api.sock")
    }
}
