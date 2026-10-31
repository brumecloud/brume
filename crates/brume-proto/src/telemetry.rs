//! The messages of telemetry: log batches from the agent on the ingest socket, and log queries
//! from the engine on the query socket.

use serde::{Deserialize, Serialize};

use crate::api::LogLineDto;

/// The longest log line, in bytes. The agent cuts longer lines to it.
pub const MAX_LOG_LINE_BYTES: usize = 16 * 1024;

/// Container log lines, from the agent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogBatch {
    pub lines: Vec<RawLogLine>,
}

/// A line that a container wrote.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawLogLine {
    /// When Docker received it, in milliseconds since the Unix epoch.
    pub timestamp_ms: u64,
    pub environment_id: String,
    pub service_id: String,
    pub deployment_id: String,
    pub stream: StdStream,
    /// Lossy UTF-8, at most `MAX_LOG_LINE_BYTES`.
    pub message: String,
}

/// The output stream that a line came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StdStream {
    Stdout,
    Stderr,
}

/// The logs of an environment, from the engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogQuery {
    pub environment_id: String,
    /// Only lines from this instant on, in milliseconds since the Unix epoch.
    pub since_ms: Option<u64>,
    /// The most lines of history to send.
    pub limit: u32,
    /// After the history, keep sending new lines.
    pub follow: bool,
}

/// An answer to a `LogQuery`: lines, history first, until `End` or until the connection closes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LogStreamFrame {
    Lines(Vec<LogLineDto>),
    /// No more lines will come.
    End,
}
