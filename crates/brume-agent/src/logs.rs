//! Container logs, from Docker to telemetry. One task follows the logs of each running managed
//! container; their lines go through a bounded queue to the shipper, which sends them to
//! `telemetry_ingest_socket` in batches. A lost line is acceptable: when the queue is full, new
//! lines are dropped and counted.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bollard::Docker;
use bollard::container::LogOutput;
use bollard::query_parameters::LogsOptionsBuilder;
use brume_proto::telemetry::{
    LogBatch, MAX_LOG_CHUNK_BYTES, MAX_LOG_LINE_BYTES, RawLogLine, StdStream, chunk_by_encoded_size,
};
use futures_util::StreamExt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::net::UnixStream;
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;

use crate::docker::LogSource;
use crate::{Throttle, WARN_INTERVAL};
use brume_runtime::framed::FramedWriter;

/// The most lines waiting for the shipper.
pub const QUEUE_LINES: usize = 10_000;
/// The most lines in one batch.
pub const MAX_BATCH_LINES: usize = 200;
/// The longest a line waits for its batch to fill.
pub const BATCH_WINDOW: Duration = Duration::from_millis(250);

/// Follows the logs of the running managed containers, and feeds the shipper.
#[derive(Debug)]
pub struct LogForwarder {
    lines: mpsc::Sender<RawLogLine>,
    dropped: Arc<AtomicU64>,
    /// The follow task of each container; each returns the timestamp of its last line.
    tasks: BTreeMap<String, JoinHandle<i128>>,
    /// The timestamp of the last line read from each container, in nanoseconds.
    last_seen: BTreeMap<String, i128>,
}

impl LogForwarder {
    /// Starts the shipper towards `socket`.
    pub fn start(socket: &Path) -> Self {
        let (lines, queue) = mpsc::channel(QUEUE_LINES);
        let dropped = Arc::new(AtomicU64::new(0));
        tokio::spawn(ship(socket.to_owned(), queue, Arc::clone(&dropped)));
        Self {
            lines,
            dropped,
            tasks: BTreeMap::new(),
            last_seen: BTreeMap::new(),
        }
    }

    /// Follows each running container of `sources` that is not followed yet, from the last line
    /// read from it, and forgets the containers that are gone.
    pub async fn update(&mut self, docker: &Docker, sources: &BTreeMap<String, LogSource>) {
        let finished: Vec<String> = self
            .tasks
            .iter()
            .filter(|(_, task)| task.is_finished())
            .map(|(name, _)| name.clone())
            .collect();
        for name in finished {
            if let Some(task) = self.tasks.remove(&name)
                && let Ok(last) = task.await
            {
                self.last_seen.insert(name, last);
            }
        }
        self.tasks.retain(|name, task| {
            let present = sources.contains_key(name);
            if !present {
                task.abort();
            }
            present
        });
        self.last_seen.retain(|name, _| sources.contains_key(name));

        for (name, source) in sources {
            if source.running && !self.tasks.contains_key(name) {
                let resume_after = self.last_seen.get(name).copied().unwrap_or(0);
                let task = tokio::spawn(follow(
                    docker.clone(),
                    name.clone(),
                    source.clone(),
                    resume_after,
                    self.lines.clone(),
                    Arc::clone(&self.dropped),
                ));
                self.tasks.insert(name.clone(), task);
            }
        }
    }
}

/// Reads the logs of `name` until its stream ends, skipping the lines up to `resume_after`
/// (nanoseconds) that an earlier stream already read. Returns the timestamp of the last line.
async fn follow(
    docker: Docker,
    name: String,
    source: LogSource,
    resume_after: i128,
    lines: mpsc::Sender<RawLogLine>,
    dropped: Arc<AtomicU64>,
) -> i128 {
    // Docker takes whole seconds: the lines of that second are read again, then skipped.
    let since_secs = i32::try_from(resume_after.div_euclid(NANOS_PER_SEC)).unwrap_or(0);
    let options = LogsOptionsBuilder::new()
        .follow(true)
        .stdout(true)
        .stderr(true)
        .timestamps(true)
        .since(since_secs)
        .build();
    let mut stream = docker.logs(&name, Some(options));
    let mut last = resume_after;
    while let Some(item) = stream.next().await {
        let Ok(output) = item else {
            break;
        };
        let (std_stream, frame) = match &output {
            LogOutput::StdOut { message } | LogOutput::Console { message } => {
                (StdStream::Stdout, message)
            }
            LogOutput::StdErr { message } => (StdStream::Stderr, message),
            LogOutput::StdIn { .. } => continue,
        };
        let Some((nanos, text)) = parse_line(frame) else {
            dropped.fetch_add(1, Ordering::Relaxed);
            continue;
        };
        if nanos <= resume_after {
            continue;
        }
        last = last.max(nanos);
        let line = RawLogLine {
            timestamp_ms: u64::try_from(nanos.div_euclid(NANOS_PER_MILLI)).unwrap_or(0),
            environment_id: source.environment_id.clone(),
            service_id: source.service_id.clone(),
            deployment_id: source.deployment_id.clone(),
            stream: std_stream,
            message: message(text),
        };
        if lines.try_send(line).is_err() {
            dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    last
}

const NANOS_PER_SEC: i128 = 1_000_000_000;
const NANOS_PER_MILLI: i128 = 1_000_000;

/// One line of `docker logs --timestamps`: when Docker received it, in nanoseconds since the
/// Unix epoch, and the line without its timestamp nor its final newline.
pub(crate) fn parse_line(frame: &[u8]) -> Option<(i128, &[u8])> {
    let space = frame.iter().position(|byte| *byte == b' ')?;
    let timestamp = std::str::from_utf8(&frame[..space]).ok()?;
    let nanos = OffsetDateTime::parse(timestamp, &Rfc3339)
        .ok()?
        .unix_timestamp_nanos();
    let text = &frame[space + 1..];
    Some((nanos, text.strip_suffix(b"\n").unwrap_or(text)))
}

/// The message of a line: lossy UTF-8, cut to `MAX_LOG_LINE_BYTES` on a character boundary.
pub(crate) fn message(bytes: &[u8]) -> String {
    let mut text = String::from_utf8_lossy(bytes).into_owned();
    if text.len() > MAX_LOG_LINE_BYTES {
        let mut end = MAX_LOG_LINE_BYTES;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    text
}

/// Sends the lines of `lines` to telemetry on `socket`, in batches of at most
/// `MAX_BATCH_LINES` lines or `BATCH_WINDOW`, until `lines` closes. A batch that cannot be sent
/// is dropped and counted in `dropped`, which only grows.
pub async fn ship(socket: PathBuf, mut lines: mpsc::Receiver<RawLogLine>, dropped: Arc<AtomicU64>) {
    let mut connection: Option<FramedWriter<OwnedWriteHalf>> = None;
    let mut unreachable = Throttle::new(WARN_INTERVAL);
    let mut drops = Throttle::new(WARN_INTERVAL);
    let mut reported = 0;
    while let Some(first) = lines.recv().await {
        let mut batch = vec![first];
        let deadline = Instant::now() + BATCH_WINDOW;
        while batch.len() < MAX_BATCH_LINES {
            match tokio::time::timeout_at(deadline, lines.recv()).await {
                Ok(Some(line)) => batch.push(line),
                Ok(None) | Err(_) => break,
            }
        }
        let count = u64::try_from(batch.len()).unwrap_or(u64::MAX);
        if let Err(error) = send_batch(&socket, &mut connection, batch).await {
            connection = None;
            dropped.fetch_add(count, Ordering::Relaxed);
            if unreachable.ready() {
                tracing::warn!(%error, "telemetry is unreachable, log lines are dropped");
            }
        }
        let total = dropped.load(Ordering::Relaxed);
        if total > reported && drops.ready() {
            tracing::warn!(lines = total - reported, "log lines were dropped");
            reported = total;
        }
    }
}

/// Sends `batch`, split so that no frame exceeds `MAX_FRAME`, connecting first if needed.
async fn send_batch(
    socket: &Path,
    connection: &mut Option<FramedWriter<OwnedWriteHalf>>,
    batch: Vec<RawLogLine>,
) -> std::io::Result<()> {
    let writer = match connection {
        Some(writer) => writer,
        None => {
            let (_, write) = UnixStream::connect(socket).await?.into_split();
            connection.insert(FramedWriter::new(write))
        }
    };
    for lines in chunk_by_encoded_size(batch, MAX_LOG_CHUNK_BYTES) {
        writer.write(&LogBatch { lines }).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    use brume_proto::frame::MAX_FRAME;
    use brume_proto::telemetry::{LogBatch, MAX_LOG_LINE_BYTES, RawLogLine, StdStream};
    use tokio::io::AsyncReadExt;
    use tokio::net::UnixListener;
    use tokio::sync::mpsc;
    use tokio::time::timeout;

    use super::{message, parse_line, ship};
    use brume_runtime::framed::FramedReader;

    const WAIT: Duration = Duration::from_secs(5);

    fn line(timestamp_ms: u64, message: &str) -> RawLogLine {
        RawLogLine {
            timestamp_ms,
            environment_id: "env".to_owned(),
            service_id: "svc".to_owned(),
            deployment_id: "dep".to_owned(),
            stream: StdStream::Stdout,
            message: message.to_owned(),
        }
    }

    #[test]
    fn docker_timestamps_keep_their_nanoseconds() {
        let (nanos, rest) =
            parse_line(b"2026-10-31T12:34:56.123456789Z GET / 200\n").expect("parses");
        assert_eq!(nanos, 1_793_450_096_123_456_789);
        assert_eq!(rest, b"GET / 200");
        let (nanos, rest) = parse_line(b"2026-10-31T12:34:56Z \n").expect("parses");
        assert_eq!(nanos, 1_793_450_096_000_000_000);
        assert_eq!(rest, b"");
        // Only the last newline goes; carriage returns and spaces stay.
        let (_, rest) = parse_line(b"2026-10-31T12:34:56Z  a\r\n\n").expect("parses");
        assert_eq!(rest, b" a\r\n");
        assert!(parse_line(b"not a timestamp\n").is_none());
        assert!(parse_line(b"").is_none());
    }

    #[test]
    fn messages_are_lossy_and_cut_on_a_char_boundary() {
        assert_eq!(message(b"caf\xc3\xa9 \xff"), "café \u{fffd}");
        let long = "é".repeat(MAX_LOG_LINE_BYTES);
        let cut = message(long.as_bytes());
        assert!(cut.len() <= MAX_LOG_LINE_BYTES);
        assert_eq!(cut.len(), MAX_LOG_LINE_BYTES);
        assert!(cut.chars().all(|c| c == 'é'));
        let odd = format!("a{long}");
        assert_eq!(message(odd.as_bytes()).len(), MAX_LOG_LINE_BYTES - 1);
    }

    /// Reads `count` lines of batches from the fake telemetry, and the batch sizes.
    async fn receive(
        reader: &mut FramedReader<tokio::net::UnixStream>,
        count: usize,
    ) -> (Vec<RawLogLine>, Vec<usize>) {
        let mut lines = Vec::new();
        let mut sizes = Vec::new();
        while lines.len() < count {
            let batch = timeout(WAIT, reader.read::<LogBatch>())
                .await
                .expect("a batch")
                .expect("decodes")
                .expect("not closed");
            sizes.push(batch.lines.len());
            lines.extend(batch.lines);
        }
        (lines, sizes)
    }

    #[tokio::test]
    async fn lines_go_in_batches_of_200_in_order() {
        let dir = tempfile::tempdir().expect("temp dir");
        let socket = dir.path().join("ingest.sock");
        let listener = UnixListener::bind(&socket).expect("binds");
        let (lines_tx, lines_rx) = mpsc::channel(super::QUEUE_LINES);
        let dropped = Arc::new(AtomicU64::new(0));
        let shipper = tokio::spawn(ship(socket.clone(), lines_rx, Arc::clone(&dropped)));
        for index in 0..450 {
            lines_tx
                .try_send(line(index, &format!("line {index}")))
                .expect("queues");
        }
        let (stream, _) = timeout(WAIT, listener.accept())
            .await
            .expect("connects")
            .expect("accepts");
        let mut reader = FramedReader::new(stream);
        let (lines, sizes) = receive(&mut reader, 450).await;
        assert!(sizes.iter().all(|size| *size <= 200), "{sizes:?}");
        let timestamps: Vec<u64> = lines.iter().map(|line| line.timestamp_ms).collect();
        assert_eq!(timestamps, (0..450).collect::<Vec<_>>());

        // A lone line leaves after the batch window.
        lines_tx.try_send(line(1_000, "late")).expect("queues");
        let (lines, _) = receive(&mut reader, 1).await;
        assert_eq!(lines[0].message, "late");

        drop(lines_tx);
        timeout(WAIT, shipper)
            .await
            .expect("stops")
            .expect("no panic");
        assert_eq!(dropped.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn hostile_lines_never_exceed_a_frame() {
        let dir = tempfile::tempdir().expect("temp dir");
        let socket = dir.path().join("ingest.sock");
        let listener = UnixListener::bind(&socket).expect("binds");
        let (lines_tx, lines_rx) = mpsc::channel(super::QUEUE_LINES);
        let shipper = tokio::spawn(ship(socket.clone(), lines_rx, Arc::new(AtomicU64::new(0))));
        // 200 lines of control characters: 200 × 16 Kio × 6 in JSON, far over 4 Mio.
        let hostile = "\u{1}".repeat(MAX_LOG_LINE_BYTES);
        for index in 0..200 {
            lines_tx.try_send(line(index, &hostile)).expect("queues");
        }
        drop(lines_tx);
        let (mut stream, _) = timeout(WAIT, listener.accept())
            .await
            .expect("connects")
            .expect("accepts");
        let mut bytes = Vec::new();
        timeout(WAIT, stream.read_to_end(&mut bytes))
            .await
            .expect("reads")
            .expect("io");
        timeout(WAIT, shipper)
            .await
            .expect("stops")
            .expect("no panic");

        let mut frames = 0;
        let mut total = 0;
        let mut rest = &bytes[..];
        while let Some(header) = rest.first_chunk::<4>() {
            let len = usize::try_from(u32::from_be_bytes(*header)).expect("fits");
            assert!(len <= MAX_FRAME);
            let batch: LogBatch = serde_json::from_slice(&rest[4..4 + len]).expect("decodes");
            total += batch.lines.len();
            frames += 1;
            rest = &rest[4 + len..];
        }
        assert!(frames > 1);
        assert_eq!(total, 200);
    }

    #[tokio::test]
    async fn unreachable_telemetry_drops_and_counts_the_batch() {
        let dir = tempfile::tempdir().expect("temp dir");
        let socket = dir.path().join("absent.sock");
        let (lines_tx, lines_rx) = mpsc::channel(super::QUEUE_LINES);
        let dropped = Arc::new(AtomicU64::new(0));
        let shipper = tokio::spawn(ship(socket, lines_rx, Arc::clone(&dropped)));
        for index in 0..3 {
            lines_tx.try_send(line(index, "lost")).expect("queues");
        }
        drop(lines_tx);
        timeout(WAIT, shipper)
            .await
            .expect("stops")
            .expect("no panic");
        assert_eq!(dropped.load(Ordering::Relaxed), 3);
    }
}
