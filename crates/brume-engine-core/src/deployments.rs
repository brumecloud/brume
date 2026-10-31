//! Deployments as long-running operations: the agent's reports and the deadlines turn into
//! commands, and each change of the log updates the agent's desired state.

use brume_domain::{
    Command, ContainerPath, Ctx, DeploymentId, DeploymentPhase, FailureReason, ImageDigestRef,
    MachineId, RevisionId, RevisionPlan, ServiceId, Timestamp,
};
use brume_proto::agent::{
    AgentToEngine, ContainerHealth, EngineToAgent, Job, JobOutcome, MAX_FAILURE_MESSAGE_BYTES,
};
use brume_proto::api::{ApiFailure, FailureKind, ServiceStatusDto};

use crate::desired::{HTTP_HEALTHCHECK_TIMEOUT_SECS, RUNNING_GRACE_SECS, desired_state};
use crate::engine::{Actor, Engine, Output};
use crate::storage::{Storage, Transaction};

/// How long a deployment may take, from its request to its success: 5 minutes.
pub const DEPLOYMENT_TIMEOUT_MILLIS: u64 = 5 * 60 * 1000;

/// How long to wait before failing again an overdue deployment whose failure was not written.
const OVERDUE_RETRY_MILLIS: u64 = 1000;

/// Cuts a message to at most `max` bytes, on a character boundary.
pub(crate) fn truncate(message: &str, max: usize) -> String {
    let mut end = message.len().min(max);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    message[..end].to_owned()
}

impl<S: Storage> Engine<S> {
    /// The agent of the registered machine connected; any other is ignored.
    pub(crate) fn agent_connected(&mut self, machine: MachineId) {
        if self.world.machine() != Some(&machine) {
            return;
        }
        self.agent = Some(machine);
        self.forget_connection();
    }

    pub(crate) fn agent_disconnected(&mut self) {
        self.agent = None;
        self.forget_connection();
    }

    /// A new connection gets the whole desired state and every pending job again.
    fn forget_connection(&mut self) {
        self.observed.clear();
        self.sent_position = None;
        self.jobs_sent.clear();
    }

    pub(crate) fn agent_message(&mut self, ctx: &mut impl Ctx, message: AgentToEngine) {
        let Some(machine) = self.agent.clone() else {
            return;
        };
        match message {
            AgentToEngine::Hello { .. } => {
                self.sent_position = None;
                self.jobs_sent.clear();
            }
            AgentToEngine::Applied { .. } => {}
            AgentToEngine::Observed(state) => {
                self.observed = state
                    .containers
                    .into_iter()
                    .map(|container| (container.deployment_id.clone(), container))
                    .collect();
                self.react_to_observed(ctx, &machine);
            }
            AgentToEngine::JobResult { job_id, outcome } => {
                self.job_result(ctx, &machine, &job_id, outcome);
            }
        }
    }

    /// Turns what the agent sees into facts about the deployments in progress.
    fn react_to_observed(&mut self, ctx: &mut impl Ctx, machine: &MachineId) {
        let now = ctx.now();
        let mut commands = Vec::new();
        for deployment in self.world.deployments_in_progress() {
            let Some(container) = self.observed.get(deployment.id.as_str()) else {
                continue;
            };
            let deployment_id = deployment.id.clone();
            match (&deployment.phase, &container.health) {
                (
                    DeploymentPhase::Starting | DeploymentPhase::Promoting,
                    ContainerHealth::Exited { code },
                ) => {
                    commands.push(Command::DeploymentFailed {
                        deployment: deployment_id,
                        reason: FailureReason::ContainerExited { code: *code },
                    });
                }
                (DeploymentPhase::Starting, ContainerHealth::Healthy) => {
                    commands.push(Command::DeploymentHealthy {
                        deployment: deployment_id,
                    });
                }
                (DeploymentPhase::Starting, ContainerHealth::Unhealthy { reason }) => {
                    let grace_secs = match deployment
                        .revision
                        .as_ref()
                        .and_then(|revision| self.world.revision(revision))
                    {
                        Some(revision)
                            if revision.spec.port.is_some()
                                && revision.spec.healthcheck_path.is_some() =>
                        {
                            HTTP_HEALTHCHECK_TIMEOUT_SECS
                        }
                        _ => RUNNING_GRACE_SECS,
                    };
                    let grace_over = deployment.started_at.is_some_and(|started| {
                        now >= started.plus_millis(u64::from(grace_secs) * 1000)
                    });
                    if grace_over {
                        commands.push(Command::DeploymentFailed {
                            deployment: deployment_id,
                            reason: FailureReason::HealthcheckFailed {
                                reason: truncate(reason, MAX_FAILURE_MESSAGE_BYTES),
                            },
                        });
                    }
                }
                (DeploymentPhase::Promoting, _) => {
                    let slug = self
                        .world
                        .service(&deployment.service)
                        .map(|service| service.slug.to_string());
                    if slug.is_some_and(|slug| container.aliases.contains(&slug)) {
                        commands.push(Command::DeploymentPromoted {
                            deployment: deployment_id,
                        });
                    }
                }
                _ => {}
            }
        }
        for command in commands {
            // A refusal means the fact is stale: the deployment moved on meanwhile.
            let _ = self.commit(
                ctx,
                &Actor::Agent(machine.clone()),
                &command,
                Transaction::default(),
            );
        }
    }

    fn job_result(
        &mut self,
        ctx: &mut impl Ctx,
        machine: &MachineId,
        job_id: &str,
        outcome: JobOutcome,
    ) {
        let Ok(deployment) = DeploymentId::parse(job_id) else {
            return;
        };
        let deployment_id = deployment.clone();
        let command = match outcome {
            JobOutcome::ImageResolved {
                digest, volumes, ..
            } => {
                let digest = ImageDigestRef::parse(&digest);
                let volumes = volumes
                    .iter()
                    .map(|path| ContainerPath::parse(path))
                    .collect::<Result<Vec<_>, _>>();
                match (digest, volumes) {
                    (Ok(digest), Ok(volumes)) => Command::ImageResolved {
                        deployment,
                        revision: RevisionId::new(ctx),
                        digest,
                        volumes,
                    },
                    _ => Command::ImageResolutionFailed {
                        deployment,
                        message: "The agent reported an invalid digest or volume path.".to_owned(),
                    },
                }
            }
            JobOutcome::Failed { message } => Command::ImageResolutionFailed {
                deployment,
                message: truncate(&message, MAX_FAILURE_MESSAGE_BYTES),
            },
        };
        let result = self.commit(
            ctx,
            &Actor::Agent(machine.clone()),
            &command,
            Transaction::default(),
        );
        // A refusal means the deployment is no longer resolving: the result is stale. A write
        // that failed loses the result, so the job goes again at the next input.
        if let Err(ApiFailure {
            kind: FailureKind::Internal,
            ..
        }) = result
        {
            self.jobs_sent.remove(&deployment_id);
        }
    }

    /// Fails the deployments past their deadline. It also re-reads the last observation, for a
    /// healthcheck whose grace ended since; the agent reports at least every 10 seconds anyway.
    pub(crate) fn tick(&mut self, ctx: &mut impl Ctx) {
        // The shell's timer fired and is gone: `drive` asks again for the next deadline.
        self.wake_at = None;
        let now = ctx.now();
        let overdue = self
            .world
            .deployments_in_progress()
            .filter(|deployment| deadline(deployment.created_at) <= now)
            .map(|deployment| deployment.id.clone())
            .collect::<Vec<_>>();
        for deployment in overdue {
            let command = Command::DeploymentFailed {
                deployment,
                reason: FailureReason::Timeout,
            };
            let _ = self.commit(ctx, &Actor::Engine, &command, Transaction::default());
        }
        if let Some(machine) = self.agent.clone() {
            self.react_to_observed(ctx, &machine);
        }
    }

    /// After each input: resolves the reused revisions, sends the pending jobs and the new
    /// desired state, and asks for a wake-up at the next deadline.
    pub(crate) fn drive(&mut self, ctx: &mut impl Ctx, out: &mut Vec<Output>) {
        // A reused revision needs no agent: its digest is known.
        let reused = self
            .world
            .deployments_in_progress()
            .filter(|deployment| deployment.phase == DeploymentPhase::Resolving)
            .filter_map(|deployment| match &deployment.plan {
                RevisionPlan::Reuse { digest, volumes } => {
                    Some((deployment.id.clone(), digest.clone(), volumes.clone()))
                }
                RevisionPlan::Existing { .. } | RevisionPlan::Resolve { .. } => None,
            })
            .collect::<Vec<_>>();
        for (deployment, digest, volumes) in reused {
            let command = Command::ImageResolved {
                deployment,
                revision: RevisionId::new(ctx),
                digest,
                volumes,
            };
            let _ = self.commit(ctx, &Actor::Engine, &command, Transaction::default());
        }

        if self.agent.is_some() {
            let jobs = self
                .world
                .deployments_in_progress()
                .filter(|deployment| deployment.phase == DeploymentPhase::Resolving)
                .filter(|deployment| !self.jobs_sent.contains(&deployment.id))
                .filter_map(|deployment| match &deployment.plan {
                    RevisionPlan::Resolve { image } => Some((deployment.id.clone(), image.clone())),
                    RevisionPlan::Existing { .. } | RevisionPlan::Reuse { .. } => None,
                })
                .collect::<Vec<_>>();
            for (deployment, image) in jobs {
                out.push(Output::ToAgent(EngineToAgent::Job(Job::ResolveImage {
                    job_id: deployment.to_string(),
                    image: image.to_string(),
                })));
                self.jobs_sent.insert(deployment);
            }
            if self.sent_position != Some(self.position) {
                // A missing value stops the desired state; the next change retries.
                if let Ok(state) = desired_state(&self.world, &self.storage, self.position) {
                    out.push(Output::ToAgent(EngineToAgent::DesiredState(state)));
                    self.sent_position = Some(self.position);
                }
            }
        }

        // A deadline already past is one whose failure could not be written (read-only, or a
        // storage error): it is retried later, never at once.
        let now = ctx.now();
        let next = self
            .world
            .deployments_in_progress()
            .map(|deployment| deadline(deployment.created_at))
            .min()
            .map(|next| match self.wake_at {
                _ if next > now => next,
                // A retry already asked for stands: later inputs do not push it back.
                Some(asked) if asked > now => asked,
                _ => now.plus_millis(OVERDUE_RETRY_MILLIS),
            });
        if next != self.wake_at {
            if let Some(next) = next {
                out.push(Output::WakeAt(next));
            }
            self.wake_at = next;
        }
    }

    /// The status of a live service, from its deployments and the observed containers.
    pub(crate) fn service_status(&self, service: &ServiceId) -> ServiceStatusDto {
        if self.world.in_progress_deployment(service).is_some() {
            return ServiceStatusDto::Deploying;
        }
        if let Some(active) = self.world.active_deployment(service) {
            return match self.observed.get(active.id.as_str()).map(|c| &c.health) {
                Some(ContainerHealth::Healthy) => ServiceStatusDto::Online,
                Some(
                    ContainerHealth::Exited { .. }
                    | ContainerHealth::Unhealthy { .. }
                    | ContainerHealth::Missing,
                ) if self.agent.is_some() => ServiceStatusDto::Crashed,
                _ => ServiceStatusDto::Stopped,
            };
        }
        let latest = self
            .world
            .deployments_of(service)
            .max_by_key(|deployment| (deployment.created_at, deployment.id.clone()));
        match latest.map(|deployment| &deployment.phase) {
            Some(DeploymentPhase::Failed(_)) => ServiceStatusDto::Crashed,
            _ => ServiceStatusDto::Stopped,
        }
    }
}

/// When a deployment requested at `created_at` times out.
pub(crate) fn deadline(created_at: Timestamp) -> Timestamp {
    created_at.plus_millis(DEPLOYMENT_TIMEOUT_MILLIS)
}
