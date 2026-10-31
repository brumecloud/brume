//! The frame codec of `brume_proto::frame` on async streams.
//!
//! A stand-in for `brume_runtime::framed`, until that crate lands: switch to it then and delete
//! this module.

use std::io;

use brume_proto::frame::{FrameDecoder, encode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The size of each read from the stream.
const READ_CHUNK: usize = 64 * 1024;

/// Reads messages from the read half of a stream.
#[derive(Debug)]
pub struct FramedReader<R> {
    inner: R,
    decoder: FrameDecoder,
    chunk: Vec<u8>,
}

impl<R: AsyncRead + Unpin> FramedReader<R> {
    /// Wraps the read half of a stream.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            decoder: FrameDecoder::default(),
            chunk: vec![0; READ_CHUNK],
        }
    }

    /// The next message, or `None` once the peer closes the stream. A frame that is too large
    /// or does not decode is an `InvalidData` error: close the connection then.
    ///
    /// Cancel safe: bytes already read stay in the decoder.
    pub async fn next<T: DeserializeOwned>(&mut self) -> io::Result<Option<T>> {
        loop {
            if let Some(message) = self
                .decoder
                .next()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
            {
                return Ok(Some(message));
            }
            let read = self.inner.read(&mut self.chunk).await?;
            if read == 0 {
                return Ok(None);
            }
            self.decoder.push(&self.chunk[..read]);
        }
    }
}

/// Writes messages to the write half of a stream.
#[derive(Debug)]
pub struct FramedWriter<W> {
    inner: W,
}

impl<W: AsyncWrite + Unpin> FramedWriter<W> {
    /// Wraps the write half of a stream.
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Encodes `message` as one frame and writes it.
    pub async fn send<T: Serialize>(&mut self, message: &T) -> io::Result<()> {
        let frame =
            encode(message).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        self.inner.write_all(&frame).await?;
        self.inner.flush().await
    }
}
