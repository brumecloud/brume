//! The messages of telemetry: log batches from the agent on the ingest socket, and log queries
//! from the engine on the query socket.

use std::io;

use serde::{Deserialize, Serialize};

use crate::api::LogLineDto;
use crate::frame::MAX_FRAME;

/// The longest log line, in bytes. The agent cuts longer lines to it.
pub const MAX_LOG_LINE_BYTES: usize = 16 * 1024;

/// The largest JSON array of lines for one frame: `MAX_FRAME`, less room for the envelope
/// (`{"lines":…}` or `{"Lines":…}`).
pub const MAX_LOG_CHUNK_BYTES: usize = MAX_FRAME - 1024;

/// Container log lines, from the agent. A line can take six times its size in JSON (a control
/// character becomes `\u0001`), so the agent splits its lines with
/// `chunk_by_encoded_size(lines, MAX_LOG_CHUNK_BYTES)`, one batch per chunk.
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
/// Telemetry splits its lines with `chunk_by_encoded_size(lines, MAX_LOG_CHUNK_BYTES)`, one
/// `Lines` per chunk.
///
/// The engine relays these frames as they are to `brume api`, on the connection of an
/// `ApiCall::OpenLogStream`, after its `ApiReply` with `ApiValue::LogStreamOpened`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LogStreamFrame {
    Lines(Vec<LogLineDto>),
    /// No more lines will come.
    End,
}

/// Splits `items`, in order, into chunks whose JSON array takes at most `max_bytes`. An item
/// that alone takes more than `max_bytes`, or does not serialize, gets a chunk of its own, which
/// the frame encoder then refuses.
pub fn chunk_by_encoded_size<T: Serialize>(items: Vec<T>, max_bytes: usize) -> Vec<Vec<T>> {
    // The brackets of the array.
    const EMPTY_ARRAY_LEN: usize = 2;
    let mut chunks = Vec::new();
    let mut chunk = Vec::new();
    let mut chunk_len = EMPTY_ARRAY_LEN;
    for item in items {
        let item_len = encoded_len(&item);
        // A comma separates the item from the previous one.
        if !chunk.is_empty() && chunk_len.saturating_add(1).saturating_add(item_len) > max_bytes {
            chunks.push(std::mem::take(&mut chunk));
            chunk_len = EMPTY_ARRAY_LEN;
        }
        let comma_len = usize::from(!chunk.is_empty());
        chunk_len = chunk_len.saturating_add(comma_len).saturating_add(item_len);
        chunk.push(item);
    }
    if !chunk.is_empty() {
        chunks.push(chunk);
    }
    chunks
}

/// The size of `item` in JSON, or `usize::MAX` if it does not serialize.
fn encoded_len<T: Serialize>(item: &T) -> usize {
    let mut counter = ByteCounter(0);
    match serde_json::to_writer(&mut counter, item) {
        Ok(()) => counter.0,
        Err(_) => usize::MAX,
    }
}

/// Counts the bytes written to it, and keeps none.
struct ByteCounter(usize);

impl io::Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 = self.0.saturating_add(buf.len());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LogBatch, LogStreamFrame, MAX_LOG_CHUNK_BYTES, MAX_LOG_LINE_BYTES, RawLogLine, StdStream,
        chunk_by_encoded_size,
    };
    use crate::api::{ApiFailure, ApiReply, ApiValue, FailureKind, LogLevel, LogLineDto};
    use crate::frame::{FrameDecoder, encode};

    fn line(id: u64, message: &str) -> LogLineDto {
        LogLineDto {
            id: id.to_string(),
            timestamp: "2026-10-31T12:34:56.789Z".to_owned(),
            service_id: "svc".to_owned(),
            level: LogLevel::Info,
            message: message.to_owned(),
        }
    }

    #[test]
    fn stream_opens_with_a_reply_then_relays_frames() {
        let opened = ApiReply {
            id: 1,
            result: Ok(ApiValue::LogStreamOpened),
        };
        let lines = LogStreamFrame::Lines(vec![line(1, "GET / 200"), line(2, "GET /x 404")]);
        let mut bytes = encode(&opened).expect("encodes");
        bytes.extend(encode(&lines).expect("encodes"));
        bytes.extend(encode(&LogStreamFrame::End).expect("encodes"));

        let mut decoder = FrameDecoder::default();
        decoder.push(&bytes);
        assert_eq!(decoder.next::<ApiReply>().ok(), Some(Some(opened)));
        assert_eq!(decoder.next::<LogStreamFrame>().ok(), Some(Some(lines)));
        assert_eq!(
            decoder.next::<LogStreamFrame>().ok(),
            Some(Some(LogStreamFrame::End))
        );
        assert_eq!(decoder.next::<LogStreamFrame>().ok(), Some(None));
    }

    #[test]
    fn refused_stream_is_a_single_failure_reply() {
        let refused = ApiReply {
            id: 1,
            result: Err(ApiFailure {
                kind: FailureKind::Unauthorized,
                message: "Log in again.".to_owned(),
            }),
        };
        let mut decoder = FrameDecoder::default();
        decoder.push(&encode(&refused).expect("encodes"));
        assert_eq!(decoder.next::<ApiReply>().ok(), Some(Some(refused)));
        assert_eq!(decoder.next::<LogStreamFrame>().ok(), Some(None));
    }

    #[test]
    fn hostile_lines_are_chunked_into_frames_that_encode() {
        // A control character takes 6 bytes in JSON (`\u0001`): each line inflates sixfold.
        let hostile = "\u{1}".repeat(MAX_LOG_LINE_BYTES);
        let raw: Vec<RawLogLine> = (0..50)
            .map(|timestamp_ms| RawLogLine {
                timestamp_ms,
                environment_id: "env".to_owned(),
                service_id: "svc".to_owned(),
                deployment_id: "dep".to_owned(),
                stream: StdStream::Stderr,
                message: hostile.clone(),
            })
            .collect();
        let chunks = chunk_by_encoded_size(raw, MAX_LOG_CHUNK_BYTES);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(serde_json::to_vec(chunk).expect("serializes").len() <= MAX_LOG_CHUNK_BYTES);
        }
        let timestamps: Vec<u64> = chunks.iter().flatten().map(|l| l.timestamp_ms).collect();
        assert_eq!(timestamps, (0..50).collect::<Vec<_>>());
        for lines in chunks {
            assert!(encode(&LogBatch { lines }).is_ok());
        }

        let dtos: Vec<LogLineDto> = (0..50).map(|id| line(id, &hostile)).collect();
        let chunks = chunk_by_encoded_size(dtos, MAX_LOG_CHUNK_BYTES);
        assert!(chunks.len() > 1);
        for lines in chunks {
            assert!(encode(&LogStreamFrame::Lines(lines)).is_ok());
        }
    }

    #[test]
    fn chunks_fill_up_to_the_limit_and_an_oversized_item_goes_alone() {
        let strings = |items: &[&str]| items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        // `["a","b"]` is 9 bytes.
        assert_eq!(
            chunk_by_encoded_size(strings(&["a", "b"]), 9),
            vec![strings(&["a", "b"])]
        );
        assert_eq!(
            chunk_by_encoded_size(strings(&["a", "b"]), 8),
            vec![strings(&["a"]), strings(&["b"])]
        );
        let big = "x".repeat(100);
        assert_eq!(
            chunk_by_encoded_size(strings(&["a", &big, "b", "c"]), 20),
            vec![strings(&["a"]), strings(&[&big]), strings(&["b", "c"])]
        );
        assert_eq!(
            chunk_by_encoded_size(Vec::<String>::new(), 20),
            Vec::<Vec<String>>::new()
        );
    }
}
