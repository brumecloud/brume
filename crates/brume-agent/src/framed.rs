//! The frames of `brume_proto::frame` on tokio streams.
//!
//! A local stand-in for `brume_runtime::framed` (track A, Task 5), which had not landed when the
//! agent was written. Replace it with the runtime's once both are merged.

use std::io;

use brume_proto::frame::{FrameDecoder, encode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// How many bytes one read asks for.
const READ_CHUNK: usize = 64 * 1024;

/// Reads messages from a stream.
pub(crate) struct FramedReader<R> {
    inner: R,
    decoder: FrameDecoder,
    chunk: Vec<u8>,
}

impl<R: AsyncRead + Unpin> FramedReader<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner,
            decoder: FrameDecoder::default(),
            chunk: vec![0; READ_CHUNK],
        }
    }

    /// The next message, or `None` once the peer closed the stream. A frame that is too large
    /// or does not decode is an error, after which the connection should be closed.
    ///
    /// Cancel-safe: the bytes read so far stay in the decoder.
    pub(crate) async fn next<T: DeserializeOwned>(&mut self) -> io::Result<Option<T>> {
        loop {
            if let Some(message) = self
                .decoder
                .next()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
            {
                return Ok(Some(message));
            }
            let len = self.inner.read(&mut self.chunk).await?;
            if len == 0 {
                return Ok(None);
            }
            self.decoder.push(&self.chunk[..len]);
        }
    }
}

/// Writes messages to a stream.
pub(crate) struct FramedWriter<W> {
    inner: W,
}

impl<W: AsyncWrite + Unpin> FramedWriter<W> {
    pub(crate) fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Writes `message` as one frame. A message over `MAX_FRAME` is refused before any byte
    /// is written.
    pub(crate) async fn send<T: Serialize>(&mut self, message: &T) -> io::Result<()> {
        let frame =
            encode(message).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        self.inner.write_all(&frame).await?;
        self.inner.flush().await
    }
}
