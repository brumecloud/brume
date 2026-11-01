//! The ingest socket: `LogBatch`es from the agent, stored then published to live queries.

use std::sync::Arc;

use anyhow::Context as _;
use brume_proto::telemetry::LogBatch;
use brume_runtime::framed::FramedReader;
use brume_runtime::peer::{Peers, SocketRole};
use tokio::net::{UnixListener, UnixStream};

use crate::{Shared, admit};

/// Accepts the agent's connections until the listener fails.
pub(crate) async fn serve(
    listener: UnixListener,
    peers: Peers,
    shared: Arc<Shared>,
) -> anyhow::Result<()> {
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .context("cannot accept on the ingest socket")?;
        if !admit(&peers, SocketRole::TelemetryIngest, &stream) {
            continue;
        }
        let shared = Arc::clone(&shared);
        tokio::spawn(async move {
            if let Err(error) = receive(stream, shared).await {
                tracing::warn!(error = format!("{error:#}"), "ingest connection dropped");
            }
        });
    }
}

/// Stores each batch of the connection, in order, until the agent closes it. The agent gets no
/// answer.
async fn receive(stream: UnixStream, shared: Arc<Shared>) -> anyhow::Result<()> {
    let (read, _write) = stream.into_split();
    let mut reader = FramedReader::new(read);
    while let Some(batch) = reader.read::<LogBatch>().await? {
        let shared = Arc::clone(&shared);
        tokio::task::spawn_blocking(move || shared.insert(batch.lines))
            .await
            .context("the log insert panicked")??;
    }
    Ok(())
}
