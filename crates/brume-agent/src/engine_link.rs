//! The connection to the engine on `agent.sock`. The agent connects, says `Hello`, then takes
//! each desired state at a position at least as recent as the last one, and answers `Applied`.
//! Everything the agent reports goes out through the outbox.

use std::io;
use std::path::Path;
use std::time::Duration;

use brume_proto::agent::{AgentToEngine, DesiredState, EngineToAgent, Job};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, watch};

use crate::framed::{FramedReader, FramedWriter};
use crate::{Throttle, WARN_INTERVAL};

/// The wait before each new attempt to reach the engine.
pub const RETRY: Duration = Duration::from_secs(5);

/// Talks to the engine on `socket` until `outbox` closes, reconnecting `retry` after each
/// failure or disconnection. Desired states go to `desired`, jobs to `jobs`.
pub async fn run(
    socket: &Path,
    retry: Duration,
    desired: watch::Sender<Option<DesiredState>>,
    jobs: mpsc::Sender<Job>,
    mut outbox: mpsc::Receiver<AgentToEngine>,
) {
    let mut applied: Option<u64> = None;
    let mut unreachable = Throttle::new(WARN_INTERVAL);
    loop {
        match UnixStream::connect(socket).await {
            Ok(stream) => {
                tracing::info!("connected to the engine");
                let session = Session {
                    desired: &desired,
                    jobs: &jobs,
                    applied: &mut applied,
                };
                match session.serve(stream, &mut outbox).await {
                    Ended::OutboxClosed => return,
                    Ended::Disconnected(error) => {
                        tracing::warn!(%error, "lost the engine, reconnecting");
                    }
                }
            }
            Err(error) => {
                if unreachable.ready() {
                    tracing::warn!(%error, socket = %socket.display(), "the engine is unreachable");
                }
            }
        }
        tokio::time::sleep(retry).await;
    }
}

/// Why a connection ended.
enum Ended {
    /// The agent is stopping.
    OutboxClosed,
    Disconnected(io::Error),
}

/// One connection to the engine.
struct Session<'a> {
    desired: &'a watch::Sender<Option<DesiredState>>,
    jobs: &'a mpsc::Sender<Job>,
    /// The position of the last desired state taken, across connections.
    applied: &'a mut Option<u64>,
}

impl Session<'_> {
    async fn serve(
        mut self,
        stream: UnixStream,
        outbox: &mut mpsc::Receiver<AgentToEngine>,
    ) -> Ended {
        let (read, write) = stream.into_split();
        let mut reader = FramedReader::new(read);
        let mut writer = FramedWriter::new(write);
        let hello = AgentToEngine::Hello {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            applied_position: *self.applied,
        };
        if let Err(error) = writer.send(&hello).await {
            return Ended::Disconnected(error);
        }
        loop {
            tokio::select! {
                message = reader.next::<EngineToAgent>() => {
                    let message = match message {
                        Ok(Some(message)) => message,
                        Ok(None) => {
                            return Ended::Disconnected(io::ErrorKind::UnexpectedEof.into());
                        }
                        Err(error) => return Ended::Disconnected(error),
                    };
                    let reply = match message {
                        EngineToAgent::DesiredState(state) => self.take(state),
                        EngineToAgent::Job(job) => {
                            // The job queue lives as long as the agent.
                            let _ = self.jobs.send(job).await;
                            None
                        }
                    };
                    if let Some(reply) = reply
                        && let Err(error) = writer.send(&reply).await
                    {
                        return Ended::Disconnected(error);
                    }
                }
                message = outbox.recv() => {
                    let Some(message) = message else {
                        return Ended::OutboxClosed;
                    };
                    if let Err(error) = writer.send(&message).await {
                        return Ended::Disconnected(error);
                    }
                }
            }
        }
    }

    /// Takes `state` unless an earlier one is more recent, and gives the `Applied` to send.
    fn take(&mut self, state: DesiredState) -> Option<AgentToEngine> {
        let position = state.position;
        if self.applied.is_some_and(|applied| position < applied) {
            tracing::debug!(position, "ignored an older desired state");
            return None;
        }
        *self.applied = Some(position);
        self.desired.send_replace(Some(state));
        Some(AgentToEngine::Applied { position })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use brume_proto::agent::{AgentToEngine, DesiredState, EngineToAgent, Job, JobOutcome};
    use tokio::net::UnixListener;
    use tokio::sync::{mpsc, watch};
    use tokio::time::timeout;

    use crate::framed::{FramedReader, FramedWriter};

    const WAIT: Duration = Duration::from_secs(5);

    fn state(position: u64) -> DesiredState {
        DesiredState {
            position,
            networks: Vec::new(),
            volumes: Vec::new(),
            containers: Vec::new(),
        }
    }

    #[tokio::test]
    async fn says_hello_applies_newer_states_and_reconnects() {
        let dir = tempfile::tempdir().expect("temp dir");
        let socket = dir.path().join("agent.sock");
        let listener = UnixListener::bind(&socket).expect("binds");
        let (desired_tx, mut desired_rx) = watch::channel(None);
        let (jobs_tx, mut jobs_rx) = mpsc::channel(4);
        let (outbox_tx, outbox_rx) = mpsc::channel(4);
        let link = tokio::spawn({
            let socket = socket.clone();
            async move {
                super::run(
                    &socket,
                    Duration::from_millis(50),
                    desired_tx,
                    jobs_tx,
                    outbox_rx,
                )
                .await;
            }
        });

        let (stream, _) = timeout(WAIT, listener.accept())
            .await
            .expect("connects")
            .expect("accepts");
        let (read, write) = stream.into_split();
        let mut engine_in = FramedReader::new(read);
        let mut engine_out = FramedWriter::new(write);
        let hello = timeout(WAIT, engine_in.next::<AgentToEngine>())
            .await
            .expect("hello");
        assert!(matches!(
            hello.expect("reads"),
            Some(AgentToEngine::Hello {
                applied_position: None,
                ..
            })
        ));

        engine_out
            .send(&EngineToAgent::DesiredState(state(5)))
            .await
            .expect("sends");
        let applied = timeout(WAIT, engine_in.next::<AgentToEngine>())
            .await
            .expect("applied");
        assert_eq!(
            applied.expect("reads"),
            Some(AgentToEngine::Applied { position: 5 })
        );
        timeout(WAIT, desired_rx.changed())
            .await
            .expect("changes")
            .expect("open");
        assert_eq!(
            desired_rx.borrow_and_update().as_ref().map(|s| s.position),
            Some(5)
        );

        // An older state is ignored; the same position is applied again.
        engine_out
            .send(&EngineToAgent::DesiredState(state(3)))
            .await
            .expect("sends");
        engine_out
            .send(&EngineToAgent::DesiredState(state(5)))
            .await
            .expect("sends");
        let applied = timeout(WAIT, engine_in.next::<AgentToEngine>())
            .await
            .expect("applied");
        assert_eq!(
            applied.expect("reads"),
            Some(AgentToEngine::Applied { position: 5 })
        );

        // Jobs go to the job queue, and the outbox to the engine.
        let job = Job::ResolveImage {
            job_id: "job-1".to_owned(),
            image: "nginx:1.29-alpine".to_owned(),
        };
        engine_out
            .send(&EngineToAgent::Job(job.clone()))
            .await
            .expect("sends");
        assert_eq!(timeout(WAIT, jobs_rx.recv()).await.expect("job"), Some(job));
        let result = AgentToEngine::JobResult {
            job_id: "job-1".to_owned(),
            outcome: JobOutcome::Failed {
                message: "no".to_owned(),
            },
        };
        outbox_tx.send(result.clone()).await.expect("queues");
        let sent = timeout(WAIT, engine_in.next::<AgentToEngine>())
            .await
            .expect("result");
        assert_eq!(sent.expect("reads"), Some(result));

        // The engine goes away: the agent comes back with the position it applied.
        drop((engine_in, engine_out));
        let (stream, _) = timeout(WAIT, listener.accept())
            .await
            .expect("reconnects")
            .expect("accepts");
        let (read, _write) = stream.into_split();
        let mut engine_in = FramedReader::new(read);
        let hello = timeout(WAIT, engine_in.next::<AgentToEngine>())
            .await
            .expect("hello");
        assert!(matches!(
            hello.expect("reads"),
            Some(AgentToEngine::Hello {
                applied_position: Some(5),
                ..
            })
        ));

        // Closing the outbox stops the link.
        drop(outbox_tx);
        timeout(WAIT, link).await.expect("stops").expect("no panic");
    }

    #[tokio::test]
    async fn retries_until_the_engine_listens() {
        let dir = tempfile::tempdir().expect("temp dir");
        let socket = dir.path().join("agent.sock");
        let (desired_tx, _desired_rx) = watch::channel(None);
        let (jobs_tx, _jobs_rx) = mpsc::channel(4);
        let (_outbox_tx, outbox_rx) = mpsc::channel(4);
        let link = tokio::spawn({
            let socket = socket.clone();
            async move {
                super::run(
                    &socket,
                    Duration::from_millis(50),
                    desired_tx,
                    jobs_tx,
                    outbox_rx,
                )
                .await;
            }
        });
        tokio::time::sleep(Duration::from_millis(200)).await;
        let listener = UnixListener::bind(&socket).expect("binds");
        let (stream, _) = timeout(WAIT, listener.accept())
            .await
            .expect("connects")
            .expect("accepts");
        let mut engine_in = FramedReader::new(stream);
        let hello = timeout(WAIT, engine_in.next::<AgentToEngine>())
            .await
            .expect("hello");
        assert!(matches!(
            hello.expect("reads"),
            Some(AgentToEngine::Hello { .. })
        ));
        link.abort();
    }
}
