//! The calls to the engine on `api.sock`: one multiplexed connection for the calls, and one
//! connection of its own per log stream.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use brume_proto::api::{ApiCall, ApiFailure, ApiReply, ApiRequest, ApiValue, LogLineDto};
use brume_proto::telemetry::LogStreamFrame;
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{mpsc, oneshot};

use crate::framed::{FramedReader, FramedWriter};

/// An engine that does not answer within this time fails the call.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

/// Why a call got no answer from the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineError {
    /// No connection to `api.sock`, or it closed before the answer.
    Unreachable,
    /// No answer within `CALL_TIMEOUT`.
    Timeout,
    /// An answer that does not fit the call.
    Protocol,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable => f.write_str("the engine is not reachable"),
            Self::Timeout => f.write_str("the engine did not answer in time"),
            Self::Protocol => f.write_str("the engine sent an unexpected answer"),
        }
    }
}

/// The engine's answer: a value, or why the call failed.
pub type Answer = Result<ApiValue, ApiFailure>;

/// The client of the engine, shared by every request.
pub struct EngineClient {
    socket: PathBuf,
    next_id: AtomicU64,
    connection: tokio::sync::Mutex<Option<Arc<Connection>>>,
}

/// The multiplexed connection. Requests go through a channel to a task that alone writes
/// them, so a request whose HTTP client leaves never cuts a frame in half; another task routes
/// each reply to its caller by `id`.
struct Connection {
    requests: mpsc::UnboundedSender<ApiRequest>,
    pending: Arc<Mutex<Pending>>,
}

/// The callers waiting for a reply. Once `open` is false, the connection is dead.
struct Pending {
    open: bool,
    waiting: BTreeMap<u64, oneshot::Sender<ApiReply>>,
}

/// A caller's place in `Pending`, removed when the caller stops waiting, whether it got its
/// reply, timed out, or its HTTP client left.
struct Waiting<'a> {
    pending: &'a Mutex<Pending>,
    id: u64,
}

impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.waiting.remove(&self.id);
        }
    }
}

impl Pending {
    /// Marks the connection dead and fails every waiting call.
    fn close(&mut self) {
        self.open = false;
        self.waiting.clear();
    }
}

impl EngineClient {
    /// A client of the engine listening on `socket`. It connects on the first call.
    pub fn new(socket: PathBuf) -> Self {
        Self {
            socket,
            next_id: AtomicU64::new(1),
            connection: tokio::sync::Mutex::new(None),
        }
    }

    /// Sends `body` with the session token, and waits for the engine's answer.
    pub async fn call(
        &self,
        session: Option<String>,
        body: ApiCall,
    ) -> Result<Answer, EngineError> {
        let connection = self.connection().await?;
        let (sender, receiver) = oneshot::channel();
        let id = {
            let mut pending = lock(&connection.pending)?;
            if !pending.open {
                return Err(EngineError::Unreachable);
            }
            // Under the lock, so the ids go out in increasing order.
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            pending.waiting.insert(id, sender);
            if connection
                .requests
                .send(ApiRequest { id, session, body })
                .is_err()
            {
                pending.close();
                return Err(EngineError::Unreachable);
            }
            id
        };
        let _waiting = Waiting {
            pending: &connection.pending,
            id,
        };
        match tokio::time::timeout(CALL_TIMEOUT, receiver).await {
            Ok(Ok(reply)) => Ok(reply.result),
            Ok(Err(_)) => Err(EngineError::Unreachable),
            Err(_) => Err(EngineError::Timeout),
        }
    }

    /// Opens the log stream of an environment on a connection of its own. The engine answers
    /// first; after `LogStreamOpened`, `LogStream::next` reads the lines.
    pub async fn open_log_stream(
        &self,
        session: Option<String>,
        environment_id: String,
    ) -> Result<Result<LogStream, ApiFailure>, EngineError> {
        let stream = UnixStream::connect(&self.socket)
            .await
            .map_err(|_| EngineError::Unreachable)?;
        let (read, write) = stream.into_split();
        let mut reader = FramedReader::new(read);
        let mut writer = FramedWriter::new(write);
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = ApiRequest {
            id,
            session,
            body: ApiCall::OpenLogStream {
                environment_id,
                since_ms: None,
            },
        };
        writer
            .send(&request)
            .await
            .map_err(|_| EngineError::Unreachable)?;
        let reply = match tokio::time::timeout(CALL_TIMEOUT, reader.next::<ApiReply>()).await {
            Ok(Ok(Some(reply))) => reply,
            Ok(Ok(None)) => return Err(EngineError::Unreachable),
            Ok(Err(_)) => return Err(EngineError::Protocol),
            Err(_) => return Err(EngineError::Timeout),
        };
        if reply.id != id {
            return Err(EngineError::Protocol);
        }
        match reply.result {
            Ok(ApiValue::LogStreamOpened) => Ok(Ok(LogStream {
                reader,
                _writer: writer,
            })),
            Ok(_) => Err(EngineError::Protocol),
            Err(failure) => Ok(Err(failure)),
        }
    }

    /// The open connection, or a new one.
    async fn connection(&self) -> Result<Arc<Connection>, EngineError> {
        let mut slot = self.connection.lock().await;
        if let Some(connection) = slot.as_ref()
            && lock(&connection.pending)?.open
        {
            return Ok(Arc::clone(connection));
        }
        let stream = UnixStream::connect(&self.socket).await.map_err(|error| {
            tracing::warn!(%error, socket = %self.socket.display(), "cannot reach the engine");
            EngineError::Unreachable
        })?;
        let (read, write) = stream.into_split();
        let pending = Arc::new(Mutex::new(Pending {
            open: true,
            waiting: BTreeMap::new(),
        }));
        let (requests, outgoing) = mpsc::unbounded_channel();
        tokio::spawn(write_requests(
            FramedWriter::new(write),
            outgoing,
            Arc::clone(&pending),
        ));
        tokio::spawn(route_replies(FramedReader::new(read), Arc::clone(&pending)));
        let connection = Arc::new(Connection { requests, pending });
        *slot = Some(Arc::clone(&connection));
        Ok(connection)
    }
}

/// Writes the requests in order, each one whole, until the connection breaks or is replaced.
async fn write_requests(
    mut writer: FramedWriter<OwnedWriteHalf>,
    mut requests: mpsc::UnboundedReceiver<ApiRequest>,
    pending: Arc<Mutex<Pending>>,
) {
    while let Some(request) = requests.recv().await {
        if let Err(error) = writer.send(&request).await {
            tracing::warn!(%error, "cannot write to the engine");
            break;
        }
    }
    if let Ok(mut pending) = lock(&pending) {
        pending.close();
    }
}

/// Hands each reply to the caller that waits for its `id`, until the connection closes.
async fn route_replies(mut reader: FramedReader<OwnedReadHalf>, pending: Arc<Mutex<Pending>>) {
    loop {
        match reader.next::<ApiReply>().await {
            Ok(Some(reply)) => {
                let Ok(mut pending) = lock(&pending) else {
                    return;
                };
                // A caller that timed out is gone: its reply is dropped.
                if let Some(sender) = pending.waiting.remove(&reply.id) {
                    let _ = sender.send(reply);
                }
            }
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(%error, "the engine connection broke");
                break;
            }
        }
    }
    if let Ok(mut pending) = lock(&pending) {
        pending.close();
    }
}

fn lock(pending: &Mutex<Pending>) -> Result<std::sync::MutexGuard<'_, Pending>, EngineError> {
    pending.lock().map_err(|_| EngineError::Unreachable)
}

/// The lines of an open log stream. Dropping it closes the connection, which tells the engine
/// to stop.
pub struct LogStream {
    reader: FramedReader<OwnedReadHalf>,
    _writer: FramedWriter<OwnedWriteHalf>,
}

impl LogStream {
    /// The next lines, or `None` at the end of the stream, or when it breaks.
    pub async fn next(&mut self) -> Option<Vec<LogLineDto>> {
        match self.reader.next::<LogStreamFrame>().await {
            Ok(Some(LogStreamFrame::Lines(lines))) => Some(lines),
            Ok(Some(LogStreamFrame::End) | None) => None,
            Err(error) => {
                tracing::warn!(%error, "the log stream broke");
                None
            }
        }
    }
}
