//! The relay of a log stream: telemetry's frames go to `brume api` as they are, without passing
//! through the engine's core.

use std::path::Path;

use brume_proto::api::{ApiFailure, ApiReply, ApiValue, FailureKind};
use brume_proto::telemetry::LogQuery;
use brume_runtime::framed::{FramedReader, FramedWriter};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::UnixStream;

/// Answers request `request` of an accepted stream, then copies telemetry's bytes to the
/// client until either side closes. If telemetry is not there, the stream is refused.
pub(crate) async fn relay<R, W>(
    telemetry: &Path,
    request: u64,
    query: LogQuery,
    client_reader: FramedReader<R>,
    mut client_writer: FramedWriter<W>,
) -> std::io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let telemetry = match UnixStream::connect(telemetry).await {
        Ok(stream) => stream,
        Err(error) => {
            tracing::warn!(%error, "telemetry is not reachable for a log stream");
            let refused = ApiReply {
                id: request,
                result: Err(ApiFailure {
                    kind: FailureKind::Internal,
                    message: "Logs are not available right now.".to_owned(),
                }),
            };
            return client_writer.write(&refused).await;
        }
    };
    let (mut from_telemetry, to_telemetry) = telemetry.into_split();
    // Dropping the write half would shut it down: telemetry would read the end of the stream
    // and could take it for a client that left. It lives until the relay ends.
    let mut to_telemetry = FramedWriter::new(to_telemetry);
    to_telemetry.write(&query).await?;
    client_writer
        .write(&ApiReply {
            id: request,
            result: Ok(ApiValue::LogStreamOpened),
        })
        .await?;
    let mut to_client = client_writer.into_inner();
    let mut from_client = client_reader.into_inner();
    let relayed = tokio::select! {
        copied = tokio::io::copy(&mut from_telemetry, &mut to_client) => copied.map(|_| ()),
        () = closed(&mut from_client) => Ok(()),
    };
    drop(to_telemetry);
    relayed
}

/// Returns once the client closes its side; anything it sends is ignored.
async fn closed<R: AsyncRead + Unpin>(reader: &mut R) {
    let mut buffer = [0u8; 1024];
    while let Ok(read) = reader.read(&mut buffer).await {
        if read == 0 {
            return;
        }
    }
}
