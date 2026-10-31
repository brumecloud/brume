//! The agent service, the only one that talks to Docker. It applies the engine's desired
//! state through bollard, resolves images and forwards container logs to telemetry.

#![forbid(unsafe_code)]

pub mod docker;
pub mod engine_link;
mod framed;
pub mod jobs;
pub mod logs;
pub mod probe;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bollard::Docker;
use brume_agent_core::backoff::Restarts;
use brume_agent_core::health::{ProbeResults, report};
use brume_agent_core::observed::DockerSnapshot;
use brume_agent_core::plan::{Action, plan};
use brume_proto::agent::{AgentToEngine, DesiredState, ObservedState};
use futures_util::future::join_all;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinSet;
use tokio::time::{Instant, MissedTickBehavior};

use crate::docker::{DockerShell, LogSource, Outcome};
use crate::logs::LogForwarder;

/// The sockets of the agent. The binary builds it from `brume_runtime::paths::Paths`
/// (`agent_socket()`, `telemetry_ingest_socket()`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentPaths {
    /// Where the engine listens for its agent.
    pub agent_socket: PathBuf,
    /// Where telemetry listens for log batches.
    pub telemetry_ingest_socket: PathBuf,
}

/// How often the agent reconciles, besides each new desired state.
pub const RECONCILE_INTERVAL: Duration = Duration::from_secs(2);
/// The longest the engine goes without an `Observed`.
pub const OBSERVED_HEARTBEAT: Duration = Duration::from_secs(10);
/// How often an unreachable Docker or engine is logged.
pub const WARN_INTERVAL: Duration = Duration::from_secs(60);

/// The most messages waiting for the engine.
const OUTBOX: usize = 64;
/// The most jobs waiting for a task.
const JOB_QUEUE: usize = 16;

/// Runs the agent. It returns only if one of its tasks stops, which is a bug: each one loops
/// forever, through an unreachable engine, Docker or telemetry.
pub async fn run(paths: AgentPaths) -> anyhow::Result<()> {
    let (desired_tx, desired_rx) = watch::channel(None);
    let (jobs_tx, jobs_rx) = mpsc::channel(JOB_QUEUE);
    let (outbox_tx, outbox_rx) = mpsc::channel(OUTBOX);
    let (docker_tx, docker_rx) = watch::channel(None);
    let logs = LogForwarder::start(&paths.telemetry_ingest_socket);

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        engine_link::run(
            &paths.agent_socket,
            engine_link::RETRY,
            desired_tx,
            jobs_tx,
            outbox_rx,
        )
        .await;
    });
    tasks.spawn(jobs::serve(jobs_rx, docker_rx, outbox_tx.clone()));
    tasks.spawn(reconcile_forever(desired_rx, docker_tx, outbox_tx, logs));
    match tasks.join_next().await {
        Some(Ok(())) | None => anyhow::bail!("a task of the agent stopped"),
        Some(Err(error)) => Err(anyhow::Error::new(error).context("a task of the agent failed")),
    }
}

/// Reconciles every `RECONCILE_INTERVAL` and at each new desired state, once the engine sent
/// one. Shares the Docker client with the jobs, and reports to the engine when what runs
/// changes, or every `OBSERVED_HEARTBEAT`.
async fn reconcile_forever(
    mut desired: watch::Receiver<Option<DesiredState>>,
    shared_docker: watch::Sender<Option<Docker>>,
    outbox: mpsc::Sender<AgentToEngine>,
    mut logs: LogForwarder,
) {
    let mut reconciler: Option<Reconciler> = None;
    let mut unreachable = Throttle::new(WARN_INTERVAL);
    let mut last_sent: Option<(ObservedState, Instant)> = None;
    let mut tick = tokio::time::interval(RECONCILE_INTERVAL);
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = tick.tick() => {}
            changed = desired.changed() => {
                if changed.is_err() {
                    return;
                }
            }
        }
        if reconciler.is_none() {
            match docker::connect().await {
                Ok(client) => {
                    tracing::info!("connected to Docker");
                    shared_docker.send_replace(Some(client.clone()));
                    reconciler = Some(Reconciler::new(DockerShell::new(client)));
                }
                Err(error) => {
                    if unreachable.ready() {
                        tracing::warn!(%error, "Docker is unreachable");
                    }
                    continue;
                }
            }
        }
        let Some(state) = desired.borrow_and_update().clone() else {
            continue;
        };
        let Some(active) = reconciler.as_mut() else {
            continue;
        };
        match active.reconcile(&state).await {
            Ok(round) => {
                logs.update(active.shell().docker(), &round.sources).await;
                let due = last_sent.as_ref().is_none_or(|(sent, at)| {
                    *sent != round.observed || at.elapsed() >= OBSERVED_HEARTBEAT
                });
                // A full outbox means the engine is away: it gets a fresh report later.
                if due
                    && outbox
                        .try_send(AgentToEngine::Observed(round.observed.clone()))
                        .is_ok()
                {
                    last_sent = Some((round.observed, Instant::now()));
                }
            }
            Err(error) => {
                if unreachable.ready() {
                    tracing::warn!(%error, "Docker is unreachable");
                }
                reconciler = None;
                shared_docker.send_replace(None);
            }
        }
    }
}

/// One reconciliation: what Docker runs, for the engine and for the log forwarder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub observed: ObservedState,
    pub sources: BTreeMap<String, LogSource>,
}

/// Brings Docker to the desired state, one round at a time, and remembers restarts and probes
/// across rounds.
#[derive(Debug)]
pub struct Reconciler {
    shell: DockerShell,
    restarts: Restarts,
    probes: ProbeResults,
}

impl Reconciler {
    pub fn new(shell: DockerShell) -> Self {
        Self {
            shell,
            restarts: Restarts::default(),
            probes: ProbeResults::default(),
        }
    }

    pub fn shell(&self) -> &DockerShell {
        &self.shell
    }

    /// Snapshots Docker, runs the plan, and reports what runs. An action that fails is logged
    /// and retried at the next round; only a failed snapshot is an error. The probes of a plan
    /// run together.
    pub async fn reconcile(
        &mut self,
        desired: &DesiredState,
    ) -> Result<Round, bollard::errors::Error> {
        let before = self.shell.observe().await?;
        let now = now_ms();
        self.restarts.settle(&before.snapshot, now);
        self.probes.settle(&before.snapshot);
        let actions = plan(desired, &before.snapshot, &self.restarts, &self.probes, now);
        if actions.is_empty() {
            let observed = report(desired, &before.snapshot, &self.probes, &self.restarts, now);
            return Ok(Round {
                observed,
                sources: before.sources,
            });
        }

        let mut probes = Vec::new();
        for action in &actions {
            if matches!(action, Action::Probe { .. }) {
                probes.push(action);
                continue;
            }
            self.run_probes(&mut probes, &before.snapshot).await;
            let result = self.shell.execute(action).await;
            if let Action::StartContainer(name) = action {
                self.restarts.record_start(name, now_ms());
            }
            if let Err(error) = result {
                tracing::warn!(action = describe(action), %error, "a Docker action failed");
            }
        }
        self.run_probes(&mut probes, &before.snapshot).await;

        let after = self.shell.observe().await?;
        let observed = report(
            desired,
            &after.snapshot,
            &self.probes,
            &self.restarts,
            now_ms(),
        );
        Ok(Round {
            observed,
            sources: after.sources,
        })
    }

    /// Runs `pending` probes together, records their results, and empties `pending`.
    async fn run_probes(&mut self, pending: &mut Vec<&Action>, snapshot: &DockerSnapshot) {
        if pending.is_empty() {
            return;
        }
        let shell = &self.shell;
        let results = join_all(pending.iter().map(|action| shell.execute(action))).await;
        for (action, result) in pending.drain(..).zip(results) {
            let Action::Probe { container, .. } = action else {
                continue;
            };
            let started_at_ms = snapshot
                .container(container)
                .and_then(|container| container.started_at_ms);
            let success = matches!(result, Ok(Outcome::Probed { success: true }));
            if let Some(started_at_ms) = started_at_ms {
                self.probes
                    .record(container, started_at_ms, success, now_ms());
            }
        }
    }
}

/// An action, for the logs: its kind and its target, never a config (which holds secrets).
fn describe(action: &Action) -> String {
    match action {
        Action::CreateNetwork(spec) => format!("create network {}", spec.name),
        Action::CreateVolume(spec) => format!("create volume {}", spec.name),
        Action::PullImage(image) => format!("pull {image}"),
        Action::CreateContainer(config) => format!("create container {}", config.name),
        Action::StartContainer(name) => format!("start {name}"),
        Action::StopContainer { name, .. } => format!("stop {name}"),
        Action::RemoveContainer(name) => format!("remove {name}"),
        Action::Reattach {
            container, network, ..
        } => format!("reattach {container} to {network}"),
        Action::Probe { container, .. } => format!("probe {container}"),
    }
}

/// Lets a repeated warning through once per interval.
#[derive(Debug)]
pub(crate) struct Throttle {
    every: Duration,
    last: Option<Instant>,
}

impl Throttle {
    pub(crate) fn new(every: Duration) -> Self {
        Self { every, last: None }
    }

    /// Whether to log now.
    pub(crate) fn ready(&mut self) -> bool {
        let now = Instant::now();
        let ready = self
            .last
            .is_none_or(|last| now.duration_since(last) >= self.every);
        if ready {
            self.last = Some(now);
        }
        ready
    }
}

/// The time, in milliseconds since the Unix epoch.
pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}
