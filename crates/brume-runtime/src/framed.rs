//! The frame codec of `brume_proto::frame` over tokio streams.

use std::io;

use brume_proto::frame::{FrameDecoder, FrameError, encode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The size of each read from the stream.
const READ_CHUNK: usize = 64 * 1024;

fn invalid(error: FrameError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

/// Reads frames from a stream.
pub struct FramedReader<R> {
    inner: R,
    decoder: FrameDecoder,
    chunk: Vec<u8>,
}

impl<R: AsyncRead + Unpin> FramedReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            decoder: FrameDecoder::default(),
            chunk: vec![0; READ_CHUNK],
        }
    }

    /// The next message, or `None` once the other side closed. A frame that is too large or
    /// is not a `T` is an `InvalidData` error: close the connection.
    pub async fn read<T: DeserializeOwned>(&mut self) -> io::Result<Option<T>> {
        loop {
            if let Some(message) = self.decoder.next().map_err(invalid)? {
                return Ok(Some(message));
            }
            let read = self.inner.read(&mut self.chunk).await?;
            if read == 0 {
                return Ok(None);
            }
            self.decoder.push(&self.chunk[..read]);
        }
    }

    /// The stream, to read raw bytes from. Bytes already buffered are lost.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

/// Writes frames to a stream.
pub struct FramedWriter<W> {
    inner: W,
}

impl<W: AsyncWrite + Unpin> FramedWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Writes one message and flushes it.
    pub async fn write<T: Serialize>(&mut self, message: &T) -> io::Result<()> {
        let frame = encode(message).map_err(invalid)?;
        self.inner.write_all(&frame).await?;
        self.inner.flush().await
    }

    /// The stream, to write raw bytes to.
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// The stream.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use brume_proto::api::{ApiCall, ApiRequest};
    use tokio::io::AsyncWriteExt;

    use super::{FramedReader, FramedWriter};

    #[tokio::test]
    async fn frames_round_trip_over_a_socket() {
        let (left, right) = tokio::net::UnixStream::pair().expect("a socket pair");
        let mut writer = FramedWriter::new(left);
        let mut reader = FramedReader::new(right);
        let request = ApiRequest {
            id: 1,
            session: Some("token".to_owned()),
            body: ApiCall::ListProjects,
        };
        writer.write(&request).await.expect("writes");
        writer.write(&request).await.expect("writes");
        assert_eq!(
            reader.read::<ApiRequest>().await.ok().flatten(),
            Some(request.clone())
        );
        assert_eq!(
            reader.read::<ApiRequest>().await.ok().flatten(),
            Some(request)
        );
        drop(writer);
        assert_eq!(reader.read::<ApiRequest>().await.ok().flatten(), None);
    }

    #[tokio::test]
    async fn oversized_frame_is_an_error() {
        let (mut left, right) = tokio::net::UnixStream::pair().expect("a socket pair");
        let mut reader = FramedReader::new(right);
        left.write_all(&u32::MAX.to_be_bytes())
            .await
            .expect("writes");
        let error = reader.read::<ApiRequest>().await.expect_err("too large");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }
}
