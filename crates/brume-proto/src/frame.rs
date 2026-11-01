//! The frame codec of every socket: a big-endian `u32` length, then that many bytes of JSON.
//! Synchronous, without tokio: `brume-runtime` wraps it around unix sockets.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::error::Category;

/// The largest JSON body of a frame, in bytes.
pub const MAX_FRAME: usize = 4 * 1024 * 1024;

/// The size of the length prefix.
const HEADER_LEN: usize = 4;

/// A frame that cannot be encoded or decoded. The connection that sent it should be closed.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    /// The JSON body is, or is announced to be, larger than `MAX_FRAME`.
    #[error("frame of {len} bytes exceeds the limit of {MAX_FRAME} bytes")]
    TooLarge {
        /// The size of the JSON body, in bytes.
        len: usize,
    },
    /// The JSON body is not the expected message. Only the error's category and position are
    /// kept: serde_json's message can quote the frame, and the frame can carry a secret.
    #[error("frame is not the expected message: {category:?} error at column {column}")]
    Json {
        /// The kind of error: syntax, data or end of input.
        category: Category,
        /// The column of the error in the JSON body, from 1.
        column: usize,
    },
}

impl From<serde_json::Error> for FrameError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json {
            category: error.classify(),
            column: error.column(),
        }
    }
}

/// Encodes `message` as one frame.
pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, FrameError> {
    let mut frame = vec![0; HEADER_LEN];
    serde_json::to_writer(&mut frame, message)?;
    let len = frame.len() - HEADER_LEN;
    let header = match u32::try_from(len) {
        Ok(header) if len <= MAX_FRAME => header,
        _ => return Err(FrameError::TooLarge { len }),
    };
    frame[..HEADER_LEN].copy_from_slice(&header.to_be_bytes());
    Ok(frame)
}

/// Splits a byte stream into messages: `push` the bytes as they arrive, then call `next` until
/// it returns `Ok(None)`.
#[derive(Debug, Default)]
pub struct FrameDecoder {
    buffer: Vec<u8>,
}

impl FrameDecoder {
    /// Appends bytes read from the stream.
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// The next complete message, or `None` until more bytes arrive.
    ///
    /// A length over `MAX_FRAME` is an error as soon as the header arrives, without waiting for
    /// the body. A body that does not decode is dropped, so the frames after it still decode.
    #[expect(
        clippy::should_implement_trait,
        reason = "generic over the message type and fallible, so not an `Iterator`"
    )]
    pub fn next<T: DeserializeOwned>(&mut self) -> Result<Option<T>, FrameError> {
        let Some(header) = self.buffer.first_chunk::<HEADER_LEN>() else {
            return Ok(None);
        };
        let len = usize::try_from(u32::from_be_bytes(*header)).unwrap_or(usize::MAX);
        if len > MAX_FRAME {
            return Err(FrameError::TooLarge { len });
        }
        let end = HEADER_LEN + len;
        if self.buffer.len() < end {
            return Ok(None);
        }
        let message = serde_json::from_slice(&self.buffer[HEADER_LEN..end]);
        self.buffer.drain(..end);
        Ok(Some(message?))
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameDecoder, FrameError, MAX_FRAME, encode};
    use crate::api::{ApiCall, ApiRequest};

    fn requests() -> (ApiRequest, ApiRequest) {
        let first = ApiRequest {
            id: 1,
            session: None,
            body: ApiCall::ListProjects,
        };
        let second = ApiRequest {
            id: 2,
            session: Some("session-token".to_owned()),
            body: ApiCall::DeleteService {
                service_id: "a\nb=\"c\" 🚀".to_owned(),
            },
        };
        (first, second)
    }

    #[test]
    fn two_frames_in_one_buffer_round_trip() {
        let (first, second) = requests();
        let mut bytes = encode(&first).expect("encodes");
        bytes.extend(encode(&second).expect("encodes"));

        let mut decoder = FrameDecoder::default();
        decoder.push(&bytes);
        assert_eq!(decoder.next::<ApiRequest>().ok(), Some(Some(first)));
        assert_eq!(decoder.next::<ApiRequest>().ok(), Some(Some(second)));
        assert_eq!(decoder.next::<ApiRequest>().ok(), Some(None));
    }

    #[test]
    fn frame_split_across_pushes_waits_for_its_last_byte() {
        let (first, _) = requests();
        let bytes = encode(&first).expect("encodes");
        let json_len = u32::from_be_bytes(bytes[..4].try_into().expect("4 bytes"));
        assert_eq!(usize::try_from(json_len).ok(), Some(bytes.len() - 4));

        let mut decoder = FrameDecoder::default();
        for byte in &bytes[..bytes.len() - 1] {
            decoder.push(std::slice::from_ref(byte));
            assert_eq!(decoder.next::<ApiRequest>().ok(), Some(None));
        }
        decoder.push(&bytes[bytes.len() - 1..]);
        assert_eq!(decoder.next::<ApiRequest>().ok(), Some(Some(first)));
    }

    #[test]
    fn frame_announced_over_max_is_too_large() {
        let announced = u32::try_from(MAX_FRAME + 1).expect("fits in u32");
        let mut decoder = FrameDecoder::default();
        decoder.push(&announced.to_be_bytes());
        assert!(matches!(
            decoder.next::<ApiRequest>(),
            Err(FrameError::TooLarge { len }) if len == MAX_FRAME + 1
        ));
    }

    #[test]
    fn encode_refuses_a_message_over_max() {
        assert_eq!(MAX_FRAME, 4 * 1024 * 1024);
        // The quotes push the JSON two bytes over the limit.
        let message = "x".repeat(MAX_FRAME);
        assert!(
            matches!(encode(&message), Err(FrameError::TooLarge { len }) if len == MAX_FRAME + 2)
        );
        assert!(encode(&"x".repeat(MAX_FRAME - 2)).is_ok());
    }

    #[test]
    fn undecodable_frame_is_skipped_and_its_error_hides_the_content() {
        let (first, _) = requests();
        let mut decoder = FrameDecoder::default();
        decoder.push(&encode(&"s3cr3t").expect("encodes"));
        decoder.push(&encode(&first).expect("encodes"));

        match decoder.next::<ApiRequest>() {
            Err(error @ FrameError::Json { .. }) => assert!(!error.to_string().contains("s3cr3t")),
            other => panic!("expected a JSON error, got {other:?}"),
        }
        assert_eq!(decoder.next::<ApiRequest>().ok(), Some(Some(first)));
    }
}
