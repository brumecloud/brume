//! The logs table, in a DuckDB file hardened as `archi/base/09-observabilite.md` requires:
//! no access outside the database file, no extension loading, and settings locked.

use std::path::Path;
use std::sync::Arc;

use anyhow::Context as _;
use brume_proto::api::{LogLevel, LogLineDto};
use brume_proto::telemetry::{MAX_LOG_LINE_BYTES, RawLogLine, StdStream};
use duckdb::{Connection, InterruptHandle, params};
use time::OffsetDateTime;
use time::macros::format_description;

/// Run once, right after opening. `lock_configuration` comes last: no `SET` works after it.
const HARDENING: &str = "SET enable_external_access = false;
SET autoinstall_known_extensions = false;
SET autoload_known_extensions = false;
SET lock_configuration = true;";

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS logs (
    id UBIGINT,
    ts_ms UBIGINT,
    environment_id VARCHAR,
    service_id VARCHAR,
    deployment_id VARCHAR,
    stream VARCHAR,
    level VARCHAR,
    message VARCHAR
);";

/// The latest `limit` lines that match, then sorted back into chronological order.
const HISTORY: &str = "SELECT id, ts_ms, environment_id, service_id, deployment_id, stream, level,
    message
FROM (
    SELECT * FROM logs
    WHERE environment_id = ? AND ts_ms >= ? AND id > ?
    ORDER BY ts_ms DESC, id DESC
    LIMIT ?
)
ORDER BY ts_ms, id";

/// A stored log line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredLine {
    /// Increases with each line stored, from 1.
    pub id: u64,
    /// When Docker received it, in milliseconds since the Unix epoch.
    pub timestamp_ms: u64,
    pub environment_id: String,
    pub service_id: String,
    pub deployment_id: String,
    pub stream: StdStream,
    pub level: LogLevel,
    pub message: String,
}

impl StoredLine {
    /// The line as the panel reads it.
    pub fn to_dto(&self) -> LogLineDto {
        LogLineDto {
            id: self.id.to_string(),
            timestamp: rfc3339(self.timestamp_ms),
            service_id: self.service_id.clone(),
            level: self.level,
            message: self.message.clone(),
        }
    }
}

/// Which lines of an environment `history` returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryQuery {
    pub environment_id: String,
    /// Only lines from this instant on, in milliseconds since the Unix epoch.
    pub since_ms: Option<u64>,
    /// Only lines with a greater id.
    pub after_id: Option<u64>,
    /// The most lines to return: the latest ones.
    pub limit: u32,
}

/// The writer of the logs table. One per database file: it assigns the ids.
pub struct Store {
    connection: Connection,
    next_id: u64,
}

impl Store {
    /// Opens or creates the database at `path`, hardens it, then creates the table.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let connection = Connection::open(path)
            .with_context(|| format!("cannot open the log database {}", path.display()))?;
        connection
            .execute_batch(HARDENING)
            .context("cannot harden the log database")?;
        connection
            .execute_batch(SCHEMA)
            .context("cannot create the logs table")?;
        let last_id: u64 = connection
            .query_row("SELECT coalesce(max(id), 0) FROM logs", [], |row| {
                row.get(0)
            })
            .context("cannot read the last log id")?;
        Ok(Self {
            connection,
            next_id: last_id + 1,
        })
    }

    /// Stores `lines` in one append, in order, with increasing ids, and returns them. A message
    /// longer than `MAX_LOG_LINE_BYTES` is cut.
    pub fn insert(&mut self, lines: Vec<RawLogLine>) -> anyhow::Result<Vec<StoredLine>> {
        let mut next_id = self.next_id;
        let stored: Vec<StoredLine> = lines
            .into_iter()
            .map(|line| {
                let id = next_id;
                next_id += 1;
                stored_line(id, line)
            })
            .collect();
        let mut appender = self
            .connection
            .appender("logs")
            .context("cannot open the logs appender")?;
        for line in &stored {
            appender
                .append_row(params![
                    line.id,
                    line.timestamp_ms,
                    line.environment_id,
                    line.service_id,
                    line.deployment_id,
                    stream_name(line.stream),
                    level_name(line.level),
                    line.message,
                ])
                .context("cannot append a log line")?;
        }
        appender.flush().context("cannot write the log lines")?;
        self.next_id = next_id;
        Ok(stored)
    }

    /// The lines of `query`, in chronological order.
    pub fn history(&self, query: &HistoryQuery) -> anyhow::Result<Vec<StoredLine>> {
        history(&self.connection, query)
    }

    /// Deletes the lines older than `cutoff_ms`, and returns how many.
    pub fn delete_before(&self, cutoff_ms: u64) -> anyhow::Result<usize> {
        self.connection
            .execute("DELETE FROM logs WHERE ts_ms < ?", params![cutoff_ms])
            .context("cannot delete old log lines")
    }

    /// A second connection to the same database, for a query that `Reader::interrupt_handle`
    /// can stop without blocking the writer.
    pub fn reader(&self) -> anyhow::Result<Reader> {
        let connection = self
            .connection
            .try_clone()
            .context("cannot open a reader on the log database")?;
        Ok(Reader { connection })
    }

    /// Runs SQL that the caller wrote, never one built from a request. For tests of the
    /// hardening.
    #[doc(hidden)]
    pub fn execute_batch(&self, sql: &str) -> anyhow::Result<()> {
        Ok(self.connection.execute_batch(sql)?)
    }
}

/// A read-only use of the log database, on a connection of its own.
pub struct Reader {
    connection: Connection,
}

impl Reader {
    /// The lines of `query`, in chronological order.
    pub fn history(&self, query: &HistoryQuery) -> anyhow::Result<Vec<StoredLine>> {
        history(&self.connection, query)
    }

    /// Stops the query running on this connection, from another thread.
    pub fn interrupt_handle(&self) -> Arc<InterruptHandle> {
        self.connection.interrupt_handle()
    }

    /// Runs SQL that the caller wrote, never one built from a request. For tests of the
    /// hardening.
    #[doc(hidden)]
    pub fn execute_batch(&self, sql: &str) -> anyhow::Result<()> {
        Ok(self.connection.execute_batch(sql)?)
    }
}

fn history(connection: &Connection, query: &HistoryQuery) -> anyhow::Result<Vec<StoredLine>> {
    let mut statement = connection
        .prepare(HISTORY)
        .context("cannot prepare the log query")?;
    let rows = statement
        .query_map(
            params![
                query.environment_id,
                query.since_ms.unwrap_or(0),
                query.after_id.unwrap_or(0),
                query.limit,
            ],
            |row| {
                Ok((
                    row.get::<_, u64>(0)?,
                    row.get::<_, u64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .context("cannot run the log query")?;
    let mut lines = Vec::new();
    for row in rows {
        let (id, timestamp_ms, environment_id, service_id, deployment_id, stream, level, message) =
            row.context("cannot read a log line")?;
        lines.push(StoredLine {
            id,
            timestamp_ms,
            environment_id,
            service_id,
            deployment_id,
            stream: parse_stream(&stream)?,
            level: parse_level(&level)?,
            message,
        });
    }
    Ok(lines)
}

fn stored_line(id: u64, line: RawLogLine) -> StoredLine {
    let mut message = line.message;
    truncate_on_char_boundary(&mut message, MAX_LOG_LINE_BYTES);
    StoredLine {
        id,
        timestamp_ms: line.timestamp_ms,
        level: crate::level::classify(line.stream, &message),
        environment_id: line.environment_id,
        service_id: line.service_id,
        deployment_id: line.deployment_id,
        stream: line.stream,
        message,
    }
}

fn truncate_on_char_boundary(text: &mut String, max_bytes: usize) {
    if text.len() <= max_bytes {
        return;
    }
    let mut end = max_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
}

/// `timestamp_ms` in RFC 3339, in UTC with milliseconds. An instant beyond the year 9999 reads
/// as the epoch: Docker never sends one.
fn rfc3339(timestamp_ms: u64) -> String {
    let format =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
    let nanos = i128::from(timestamp_ms) * 1_000_000;
    OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .ok()
        .and_then(|instant| instant.format(&format).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_owned())
}

fn stream_name(stream: StdStream) -> &'static str {
    match stream {
        StdStream::Stdout => "stdout",
        StdStream::Stderr => "stderr",
    }
}

fn parse_stream(name: &str) -> anyhow::Result<StdStream> {
    match name {
        "stdout" => Ok(StdStream::Stdout),
        "stderr" => Ok(StdStream::Stderr),
        _ => anyhow::bail!("unknown stream {name:?} in the log database"),
    }
}

fn level_name(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    }
}

fn parse_level(name: &str) -> anyhow::Result<LogLevel> {
    match name {
        "info" => Ok(LogLevel::Info),
        "warn" => Ok(LogLevel::Warn),
        "error" => Ok(LogLevel::Error),
        _ => anyhow::bail!("unknown level {name:?} in the log database"),
    }
}
