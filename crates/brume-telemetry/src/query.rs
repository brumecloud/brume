//! The query socket: one `LogQuery` per connection from the engine, answered with
//! `LogStreamFrame`s, the history first, then the new lines if the query follows.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use brume_proto::telemetry::{
    LogQuery, LogStreamFrame, MAX_LOG_CHUNK_BYTES, chunk_by_encoded_size,
};
use tokio::net::unix::OwnedWriteHalf;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast::error::{RecvError, TryRecvError};

use crate::Shared;
use crate::framed::{FramedReader, FramedWriter};
use crate::store::{HistoryQuery, StoredLine};

/// The most lines of history that one query gets.
const MAX_HISTORY_LINES: u32 = 5_000;

/// A DuckDB query that runs longer is interrupted.
const QUERY_TIMEOUT: Duration = Duration::from_secs(2);

/// Accepts engine connections until the listener fails.
pub(crate) async fn serve(listener: UnixListener, shared: Arc<Shared>) -> anyhow::Result<()> {
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .context("cannot accept on the query socket")?;
        let shared = Arc::clone(&shared);
        tokio::spawn(async move {
            if let Err(error) = answer(stream, shared).await {
                tracing::warn!(error = format!("{error:#}"), "log query dropped");
            }
        });
    }
}

/// Reads the query, sends the history, then follows the new lines until the engine closes the
/// connection.
async fn answer(stream: UnixStream, shared: Arc<Shared>) -> anyhow::Result<()> {
    let (read, write) = stream.into_split();
    let mut reader = FramedReader::new(read);
    let mut writer = FramedWriter::new(write);
    let Some(query) = reader.next::<LogQuery>().await? else {
        return Ok(());
    };
    // Subscribe before reading the history: a line stored in between comes from one or the
    // other, and the ids drop the duplicates.
    let live = query.follow.then(|| shared.subscribe());
    let mut history = HistoryQuery {
        environment_id: query.environment_id.clone(),
        since_ms: query.since_ms,
        after_id: None,
        limit: query.limit.min(MAX_HISTORY_LINES),
    };
    let lines = read_history(&shared, history.clone()).await?;
    let mut last_id = lines.iter().map(|line| line.id).max().unwrap_or(0);
    send_lines(&mut writer, &lines).await?;
    let Some(mut live) = live else {
        writer.send(&LogStreamFrame::End).await?;
        return Ok(());
    };

    loop {
        let first = tokio::select! {
            received = live.recv() => received,
            // The engine sends nothing after the query: any frame, or the end, closes the stream.
            _ = reader.next::<LogQuery>() => return Ok(()),
        };
        let mut pending = Vec::new();
        let mut lagged = false;
        let mut closed = false;
        match first {
            Ok(line) => pending.push(line),
            Err(RecvError::Lagged(_)) => lagged = true,
            Err(RecvError::Closed) => closed = true,
        }
        while !lagged && !closed {
            match live.try_recv() {
                Ok(line) => pending.push(line),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Lagged(_)) => lagged = true,
                Err(TryRecvError::Closed) => closed = true,
            }
        }
        let fresh: Vec<StoredLine> = pending
            .into_iter()
            .filter(|line| {
                line.id > last_id
                    && line.environment_id == query.environment_id
                    && line.timestamp_ms >= query.since_ms.unwrap_or(0)
            })
            .collect();
        last_id = fresh.iter().map(|line| line.id).max().unwrap_or(last_id);
        send_lines(&mut writer, &fresh).await?;
        if lagged {
            // The channel dropped lines for this slow reader: take them from the database.
            tracing::warn!("a log follower lagged, catching up from the database");
            history.after_id = Some(last_id);
            history.limit = MAX_HISTORY_LINES;
            let missed = read_history(&shared, history.clone()).await?;
            last_id = missed.iter().map(|line| line.id).max().unwrap_or(last_id);
            send_lines(&mut writer, &missed).await?;
        }
        if closed {
            writer.send(&LogStreamFrame::End).await?;
            return Ok(());
        }
    }
}

/// Runs `query` on a connection of its own, and interrupts it after `QUERY_TIMEOUT`.
async fn read_history(shared: &Shared, query: HistoryQuery) -> anyhow::Result<Vec<StoredLine>> {
    let reader = shared.reader()?;
    let interrupt = reader.interrupt_handle();
    let task = tokio::task::spawn_blocking(move || reader.history(&query));
    match tokio::time::timeout(QUERY_TIMEOUT, task).await {
        Ok(joined) => joined.context("the log query panicked")?,
        Err(_) => {
            interrupt.interrupt();
            anyhow::bail!("the log query ran longer than {QUERY_TIMEOUT:?}")
        }
    }
}

/// Sends `lines` as `Lines` frames that each fit in a frame.
async fn send_lines(
    writer: &mut FramedWriter<OwnedWriteHalf>,
    lines: &[StoredLine],
) -> io::Result<()> {
    if lines.is_empty() {
        return Ok(());
    }
    let dtos = lines.iter().map(StoredLine::to_dto).collect();
    for chunk in chunk_by_encoded_size(dtos, MAX_LOG_CHUNK_BYTES) {
        writer.send(&LogStreamFrame::Lines(chunk)).await?;
    }
    Ok(())
}
