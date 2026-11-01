//! The agent service, the only one that talks to Docker. It applies the engine's desired
//! state through bollard, runs the public entry (cloudflared and Traefik) and writes Traefik's
//! config, resolves images and forwards container logs to telemetry.

#![forbid(unsafe_code)]

pub mod docker;
pub mod engine_link;
pub mod jobs;
pub mod logs;
pub mod probe;
pub mod traefik;

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bollard::Docker;
use bollard::errors::Error;
use brume_agent_core::backoff::Restarts;
use brume_agent_core::config::Component;
use brume_agent_core::health::{ProbeResults, report};
use brume_agent_core::ingress::{
    CLOUDFLARED_CONTAINER, TRAEFIK_CONTAINER, traefik_dynamic_config, valid_routes,
};
use brume_agent_core::observed::{DockerSnapshot, SnapState};
use brume_agent_core::plan::{Action, Refusal, Resource, plan, refusals};
use brume_proto::agent::{AgentToEngine, DesiredState, ObservedState};
use brume_runtime::paths::Paths;
use futures_util::FutureExt;
use futures_util::future::join_all;
use tokio::sync::{mpsc, watch};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::{Instant, MissedTickBehavior};

use crate::docker::{DockerShell, LogSource, Observation, Outcome};
use crate::logs::LogForwarder;
use crate::probe::PROBE_IMAGE;

/// How often the agent reconciles, besides each new desired state.
pub const RECONCILE_INTERVAL: Duration = Duration::from_secs(2);
/// The longest the engine goes without an `Observed`.
pub const OBSERVED_HEARTBEAT: Duration = Duration::from_secs(10);
/// The wait before an image is pulled again, after a pull that ended.
pub const PULL_RETRY: Duration = Duration::from_secs(30);
/// How often an unreachable Docker or engine is logged.
pub const WARN_INTERVAL: Duration = Duration::from_secs(60);

/// The most messages waiting for the engine.
const OUTBOX: usize = 64;
/// The most jobs waiting for a task.
const JOB_QUEUE: usize = 16;

/// Runs the agent. It returns only if one of its tasks stops, which is a bug: each one loops
/// forever, through an unreachable engine, Docker or telemetry.
pub async fn run(paths: Paths) -> anyhow::Result<()> {
    let (desired_tx, desired_rx) = watch::channel(None);
    let (jobs_tx, jobs_rx) = mpsc::channel(JOB_QUEUE);
    let (outbox_tx, outbox_rx) = mpsc::channel(OUTBOX);
    let (docker_tx, docker_rx) = watch::channel(None);
    let logs = LogForwarder::start(&paths.telemetry_ingest_socket());
    let traefik_dir = paths.traefik_dir();

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        engine_link::run(
            &paths.agent_socket(),
            engine_link::RETRY,
            desired_tx,
            jobs_tx,
            outbox_rx,
        )
        .await;
    });
    tasks.spawn(jobs::serve(jobs_rx, docker_rx, outbox_tx.clone()));
    tasks.spawn(reconcile_forever(
        desired_rx,
        docker_tx,
        outbox_tx,
        logs,
        traefik_dir,
    ));
    match tasks.join_next().await {
        Some(Ok(())) | None => anyhow::bail!("a task of the agent stopped"),
        Some(Err(error)) => Err(anyhow::Error::new(error).context("a task of the agent failed")),
    }
}

/// Reconciles every `RECONCILE_INTERVAL` and at each new desired state, once the engine sent
/// one. Shares the Docker client with the jobs, and reports to the engine when what runs
/// changes, or every `OBSERVED_HEARTBEAT`. Writes Traefik's config in `traefik_dir`.
async fn reconcile_forever(
    mut desired: watch::Receiver<Option<DesiredState>>,
    shared_docker: watch::Sender<Option<Docker>>,
    outbox: mpsc::Sender<AgentToEngine>,
    mut logs: LogForwarder,
    traefik_dir: PathBuf,
) {
    let mut reconciler: Option<Reconciler> = None;
    let mut connected = false;
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
        if !connected {
            match docker::connect().await {
                Ok(client) => {
                    tracing::info!("connected to Docker");
                    shared_docker.send_replace(Some(client.clone()));
                    let shell = DockerShell::new(client).with_traefik_dir(traefik_dir.clone());
                    if let Err(error) = shell.sweep_probes().await {
                        tracing::warn!(%error, "cannot remove leftover probe containers");
                    }
                    match reconciler.as_mut() {
                        Some(active) => active.set_shell(shell),
                        None => reconciler = Some(Reconciler::new(shell)),
                    }
                    connected = true;
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
                connected = false;
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

/// Brings Docker to the desired state, one round at a time, and remembers restarts, probes,
/// image pulls and warnings across rounds.
#[derive(Debug)]
pub struct Reconciler {
    shell: DockerShell,
    restarts: Restarts,
    probes: ProbeResults,
    /// The pulls in flight, by image. They run beside the rounds, which skip the containers
    /// that wait for them: a slow registry never blocks a round.
    pulls: BTreeMap<String, JoinHandle<Result<(), bollard::errors::Error>>>,
    /// When the last pull of each image ended, and whether it succeeded, for `PULL_RETRY`.
    pulled: BTreeMap<String, (Instant, bool)>,
    /// When each failing action was last logged, so that each is logged once per interval.
    warned: BTreeMap<String, Instant>,
    /// The networks and volumes that `plan` refuses, logged once each.
    refused: RefusalLog,
}

impl Reconciler {
    pub fn new(shell: DockerShell) -> Self {
        Self {
            shell,
            restarts: Restarts::default(),
            probes: ProbeResults::default(),
            pulls: BTreeMap::new(),
            pulled: BTreeMap::new(),
            warned: BTreeMap::new(),
            refused: RefusalLog::default(),
        }
    }

    pub fn shell(&self) -> &DockerShell {
        &self.shell
    }

    /// Swaps the Docker client after a reconnection. Restarts and probes are kept; the pulls
    /// of the old client stop.
    pub fn set_shell(&mut self, shell: DockerShell) {
        for task in self.pulls.values() {
            task.abort();
        }
        self.pulls.clear();
        self.shell = shell;
    }

    /// Snapshots Docker, runs the plan, and reports what runs. An action that fails is logged
    /// and retried at the next round; only a failed snapshot is an error. Image pulls start in
    /// the background, and the probes of a plan run together. Traefik's config is written
    /// first: Traefik is created only once it is on disk.
    pub async fn reconcile(&mut self, desired: &DesiredState) -> Result<Round, Error> {
        self.reap_pulls();
        let traefik_ready = self.write_traefik_config(desired);
        let before = self.shell.observe().await?;
        self.warn_unreadable(&before);
        self.warn_exited_components(desired, &before.snapshot);
        let now = now_ms();
        self.restarts.settle(&before.snapshot, now);
        self.probes.settle(&before.snapshot);
        self.warn_refusals(desired, &before.snapshot);
        let actions = plan(desired, &before.snapshot, &self.restarts, &self.probes, now);
        if actions.is_empty() {
            let observed = report(desired, &before.snapshot, &self.probes, &self.restarts, now);
            return Ok(Round {
                observed,
                sources: before.sources,
            });
        }

        let mut probes = Vec::new();
        let mut missing_images = BTreeSet::new();
        let mut held = BTreeSet::new();
        for action in &actions {
            match action {
                Action::Probe { .. } => {
                    probes.push(action);
                    continue;
                }
                Action::PullImage(image) => {
                    if !self.image_pulled(image) {
                        missing_images.insert(image.as_str());
                    }
                    continue;
                }
                Action::CreateContainer(config)
                    if missing_images.contains(config.image.as_str())
                        || (config.component == Some(Component::Traefik) && !traefik_ready) =>
                {
                    held.insert(config.name.as_str());
                    continue;
                }
                Action::StartContainer(name) if held.contains(name.as_str()) => {
                    continue;
                }
                _ => {}
            }
            self.run_probes(&mut probes, &before.snapshot, now).await;
            let result = self.shell.execute(action).await;
            if let Action::StartContainer(name) = action {
                self.restarts.record_start(name, now_ms());
            }
            if let Err(error) = result {
                self.warn(describe(action), failure(action, &error));
            }
        }
        self.run_probes(&mut probes, &before.snapshot, now).await;

        let after = self.shell.observe().await?;
        self.warn_unreadable(&after);
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

    /// Runs `pending` probes together, records their results as of `round_ms`, the start of
    /// the round, and empties `pending`. Without the probe image, it starts its pull and leaves
    /// the probes due for a later round.
    async fn run_probes(
        &mut self,
        pending: &mut Vec<&Action>,
        snapshot: &DockerSnapshot,
        round_ms: u64,
    ) {
        if pending.is_empty() {
            return;
        }
        if !self.probe_image_ready().await {
            pending.clear();
            return;
        }
        let shell = &self.shell;
        let results = join_all(pending.iter().map(|action| shell.execute(action))).await;
        let outcomes = pending
            .drain(..)
            .zip(results)
            .filter_map(|(action, result)| match action {
                Action::Probe { container, .. } => Some((
                    container.as_str(),
                    matches!(result, Ok(Outcome::Probed { success: true })),
                )),
                _ => None,
            });
        record_probes(&mut self.probes, snapshot, outcomes, round_ms);
    }

    /// Writes Traefik's config for the routes of `desired`'s entry, and whether it is on disk,
    /// so that Traefik may be created. Without an entry, there is nothing to write.
    fn write_traefik_config(&mut self, desired: &DesiredState) -> bool {
        let Some(ingress) = &desired.ingress else {
            return true;
        };
        let routes = valid_routes(&ingress.routes).len();
        let skipped = ingress.routes.len() - routes;
        if skipped > 0 {
            self.warn(
                "route the public hosts".to_owned(),
                format!("{skipped} invalid routes are left out"),
            );
        }
        let Some(dir) = self.shell.traefik_dir().map(Path::to_path_buf) else {
            self.warn(
                "write Traefik's config".to_owned(),
                "the agent has no directory for it".to_owned(),
            );
            return false;
        };
        match traefik::write_config(&dir, &traefik_dynamic_config(&ingress.routes)) {
            Ok(written) => {
                if written {
                    tracing::info!(routes, dir = %dir.display(), "wrote Traefik's config");
                }
                true
            }
            Err(error) => {
                self.warn("write Traefik's config".to_owned(), error.to_string());
                false
            }
        }
    }

    /// Logs a component of the entry that exited, at most once per `WARN_INTERVAL`: the engine
    /// does not see the components. The plan starts it again, with a backoff.
    fn warn_exited_components(&mut self, desired: &DesiredState, snapshot: &DockerSnapshot) {
        if desired.ingress.is_none() {
            return;
        }
        for name in [TRAEFIK_CONTAINER, CLOUDFLARED_CONTAINER] {
            if let Some(container) = snapshot.container(name)
                && container.managed
                && let SnapState::Exited { code } = container.state
            {
                self.warn(
                    format!("run {name}"),
                    format!("it exited with code {code}: see `docker logs {name}`"),
                );
            }
        }
    }

    /// Whether the probe image is present; if not, its pull starts.
    async fn probe_image_ready(&mut self) -> bool {
        if self.pulls.contains_key(PROBE_IMAGE) {
            return false;
        }
        if self.shell.docker().inspect_image(PROBE_IMAGE).await.is_ok() {
            return true;
        }
        self.image_pulled(PROBE_IMAGE)
    }

    /// Whether a pull of `image` succeeded in the last `PULL_RETRY`, so that Docker should have
    /// it. Otherwise its pull starts in the background, unless one runs or failed in the last
    /// `PULL_RETRY`.
    fn image_pulled(&mut self, image: &str) -> bool {
        let now = Instant::now();
        self.pulled
            .retain(|_, (ended, _)| now.duration_since(*ended) < PULL_RETRY);
        if let Some((_, succeeded)) = self.pulled.get(image) {
            return *succeeded;
        }
        if self.pulls.contains_key(image) {
            return false;
        }
        let docker = self.shell.docker().clone();
        let reference = image.to_owned();
        let task = tokio::spawn(async move { jobs::pull(&docker, &reference).await });
        self.pulls.insert(image.to_owned(), task);
        false
    }

    /// Forgets the pulls that ended, and logs those that failed.
    fn reap_pulls(&mut self) {
        let finished: Vec<String> = self
            .pulls
            .iter()
            .filter(|(_, task)| task.is_finished())
            .map(|(image, _)| image.clone())
            .collect();
        for image in finished {
            let Some(task) = self.pulls.remove(&image) else {
                continue;
            };
            let succeeded = match task.now_or_never() {
                Some(Ok(Ok(()))) => true,
                Some(Ok(Err(error))) => {
                    self.warn(format!("pull {image}"), error.to_string());
                    false
                }
                Some(Err(error)) => {
                    tracing::warn!(%error, image, "the pull of an image panicked");
                    false
                }
                None => false,
            };
            self.pulled.insert(image, (Instant::now(), succeeded));
        }
    }

    /// Logs that `what` failed with `error`, at most once per `WARN_INTERVAL` for each `what`.
    fn warn(&mut self, what: String, error: String) {
        let now = Instant::now();
        self.warned
            .retain(|_, at| now.duration_since(*at) < WARN_INTERVAL);
        if let Entry::Vacant(entry) = self.warned.entry(what) {
            tracing::warn!(action = %entry.key(), %error, "a Docker action failed");
            entry.insert(now);
        }
    }

    /// Logs the managed containers that Docker failed to inspect, left out of the round.
    fn warn_unreadable(&mut self, observation: &Observation) {
        for (name, error) in &observation.unreadable {
            self.warn(format!("inspect {name}"), error.clone());
        }
    }

    /// Logs each network or volume that `plan` refuses, once while the refusal lasts: the
    /// containers that need it stay `Missing` without any other sign.
    fn warn_refusals(&mut self, desired: &DesiredState, snapshot: &DockerSnapshot) {
        let current = refusals(desired, snapshot);
        for refusal in self.refused.fresh(&current) {
            let resource = match refusal.resource {
                Resource::Network => "network",
                Resource::Volume => "volume",
            };
            tracing::warn!(
                resource,
                name = %refusal.name,
                reason = refusal.reason.describe(),
                "Brume neither creates nor uses it: the containers that need it stay missing"
            );
        }
    }
}

/// What to log of `error`, the failure of `action`. For a container's creation, only Docker's
/// HTTP status or the kind of error: Docker may echo a variable of the body, value included,
/// as in `invalid environment variable: NAME=value`.
fn failure(action: &Action, error: &Error) -> String {
    match (action, error) {
        (Action::CreateContainer(_), Error::DockerResponseServerError { status_code, .. }) => {
            format!("Docker answered with status {status_code}")
        }
        (Action::CreateContainer(_), _) => error_kind(error),
        _ => error.to_string(),
    }
}

/// The variant of `error`, without its fields: `RequestTimeoutError`, `IOError`…
fn error_kind(error: &Error) -> String {
    let debug = format!("{error:?}");
    debug
        .split([' ', '{', '('])
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// Records the result of each probe as of `round_ms`, the start of its round: `plan` decided
/// the probes at that instant, so a container is due again one `PROBE_INTERVAL_MS` after it,
/// not after the probe ended. A container gone from `snapshot` records nothing.
fn record_probes<'a>(
    probes: &mut ProbeResults,
    snapshot: &DockerSnapshot,
    results: impl IntoIterator<Item = (&'a str, bool)>,
    round_ms: u64,
) {
    for (container, success) in results {
        let started_at_ms = snapshot
            .container(container)
            .and_then(|container| container.started_at_ms);
        if let Some(started_at_ms) = started_at_ms {
            probes.record(container, started_at_ms, success, round_ms);
        }
    }
}

/// The refusals of `plan` already logged: each is logged once while it lasts, and again if it
/// comes back after it went away.
#[derive(Debug, Default)]
struct RefusalLog {
    logged: BTreeSet<Refusal>,
}

impl RefusalLog {
    /// The refusals of `current` not logged yet, which it now remembers. It forgets those that
    /// are gone.
    fn fresh<'a>(&mut self, current: &'a [Refusal]) -> Vec<&'a Refusal> {
        self.logged.retain(|logged| current.contains(logged));
        current
            .iter()
            .filter(|refusal| self.logged.insert((*refusal).clone()))
            .collect()
    }
}

/// An action, for the logs: its kind and its target, never a config (which holds secrets).
fn describe(action: &Action) -> String {
    match action {
        Action::CreateNetwork(spec) => format!("create network {}", spec.name),
        Action::RemoveNetwork(name) => format!("remove network {name}"),
        Action::CreateVolume(spec) => format!("create volume {}", spec.name),
        Action::PullImage(image) => format!("pull {image}"),
        Action::CreateContainer(config) => format!("create container {}", config.name),
        Action::StartContainer(name) => format!("start {name}"),
        Action::StopContainer { name, .. } => format!("stop {name}"),
        Action::RemoveContainer(name) => format!("remove {name}"),
        Action::Reattach {
            container, network, ..
        } => format!("reattach {container} to {network}"),
        Action::Disconnect { container, network } => {
            format!("disconnect {container} from {network}")
        }
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
        self.ready_at(Instant::now())
    }

    /// Whether to log at `now`.
    fn ready_at(&mut self, now: Instant) -> bool {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bollard::errors::Error;
    use brume_agent_core::config::container_config;
    use brume_agent_core::health::ProbeResults;
    use brume_agent_core::observed::{DockerSnapshot, SnapContainer, SnapState};
    use brume_agent_core::plan::{Action, Refusal, RefusalReason, Resource};
    use brume_proto::agent::{ContainerSpec, EnvValue, EnvVar, Healthcheck};
    use tokio::time::Instant;

    use super::{RefusalLog, Throttle, failure, record_probes};

    #[test]
    fn create_failures_never_log_docker_message() {
        let spec = ContainerSpec {
            name: "brume-web".to_owned(),
            image: "nginx@sha256:aa".to_owned(),
            environment_id: "env".to_owned(),
            service_id: "svc".to_owned(),
            deployment_id: "dep".to_owned(),
            env: vec![EnvVar {
                name: "TOKEN".to_owned(),
                value: EnvValue::Secret("s3cr3t".into()),
            }],
            command: None,
            networks: Vec::new(),
            mounts: Vec::new(),
            cpu_millis: 100,
            memory_mb: 64,
            stop_timeout_secs: 10,
            healthcheck: Healthcheck::Running { grace_secs: 0 },
        };
        let create = Action::CreateContainer(Box::new(container_config(&spec)));
        // What Docker answers to an entry without a name.
        let echoed = Error::DockerResponseServerError {
            status_code: 400,
            message: "invalid environment variable: =s3cr3t".to_owned(),
        };
        let logged = failure(&create, &echoed);
        assert!(logged.contains("400"), "{logged}");
        assert!(!logged.contains("s3cr3t"), "{logged}");
        assert!(!logged.contains("invalid environment"), "{logged}");
        let logged = failure(&create, &Error::RequestTimeoutError);
        assert_eq!(logged, "RequestTimeoutError");

        // Other actions carry no variable: Docker's message helps.
        let start = Action::StartContainer("brume-web".to_owned());
        let refused = Error::DockerResponseServerError {
            status_code: 500,
            message: "no such image".to_owned(),
        };
        assert!(failure(&start, &refused).contains("no such image"));
    }

    #[test]
    fn probes_are_due_one_interval_after_their_round() {
        let snapshot = DockerSnapshot {
            containers: vec![SnapContainer {
                name: "brume-web".to_owned(),
                managed: true,
                deployment_id: "dep".to_owned(),
                image: "nginx@sha256:aa".to_owned(),
                state: SnapState::Running,
                started_at_ms: Some(1_000),
                networks: Vec::new(),
                mounts: Vec::new(),
                config_hash: String::new(),
                stop_timeout_secs: 10,
            }],
            ..DockerSnapshot::default()
        };
        let mut probes = ProbeResults::default();
        // The probe ends after the round's start; the round's start is what counts.
        record_probes(&mut probes, &snapshot, [("brume-web", false)], 10_000);
        assert!(probes.attempted("brume-web", 1_000));
        assert!(!probes.due("brume-web", 1_000, 11_999));
        assert!(probes.due("brume-web", 1_000, 12_000));
        // A container gone from the snapshot records nothing.
        record_probes(&mut probes, &snapshot, [("brume-gone", true)], 10_000);
        assert!(!probes.attempted("brume-gone", 1_000));
    }

    #[test]
    fn each_refusal_is_logged_once_while_it_lasts() {
        let host = Refusal {
            resource: Resource::Network,
            name: "host".to_owned(),
            reason: RefusalReason::BadName,
        };
        let foreign = Refusal {
            resource: Resource::Volume,
            name: "brume-vol-a".to_owned(),
            reason: RefusalReason::Foreign,
        };
        let mut log = RefusalLog::default();
        assert_eq!(log.fresh(std::slice::from_ref(&host)), [&host]);
        assert!(log.fresh(std::slice::from_ref(&host)).is_empty());
        let both = [host.clone(), foreign.clone()];
        assert_eq!(log.fresh(&both), [&foreign]);
        assert!(log.fresh(std::slice::from_ref(&foreign)).is_empty());
        // Gone, then back: logged again.
        assert_eq!(log.fresh(&both), [&host]);
    }

    #[test]
    fn throttle_lets_one_warning_through_per_interval() {
        let mut throttle = Throttle::new(Duration::from_secs(60));
        let start = Instant::now();
        assert!(throttle.ready_at(start));
        assert!(!throttle.ready_at(start));
        assert!(!throttle.ready_at(start + Duration::from_secs(59)));
        assert!(throttle.ready_at(start + Duration::from_secs(60)));
        assert!(!throttle.ready_at(start + Duration::from_secs(61)));
    }
}
