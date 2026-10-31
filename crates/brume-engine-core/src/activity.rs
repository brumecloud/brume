//! The activity view: the log's events as English sentences, without any value, and the
//! logs of a deployment derived from its events.

use std::collections::BTreeMap;

use brume_domain::{
    DeployReason, Deployment, DeploymentId, DeploymentPhase, EnvironmentId, Event, EventEnvelope,
    FailureReason, ProjectId, RevisionPlan, ServiceId, ServiceVariableValue, SharedValue,
    Timestamp, VarName, VariableId, VariableOwner,
};
use brume_proto::api::{
    ActivityEventDto, ApiFailure, ApiValue, EventsPageDto, FailureKind, LogLevel, LogLineDto,
    LogStream,
};

use crate::api::{not_found, parse_id};
use crate::engine::{Engine, failure, storage_failure};
use crate::storage::Storage;

/// The largest page of `ListEvents`.
pub const MAX_EVENTS_PAGE: u32 = 200;

/// Where a variable lives.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Scope {
    Shared(VariableOwner),
    Service(ServiceId),
}

/// What the activity view and the secret versions need about variables, including removed
/// ones, which the `World` forgets.
#[derive(Clone, Debug, Default)]
pub(crate) struct History {
    variables: BTreeMap<VariableId, (VarName, Scope)>,
    secret_versions: BTreeMap<VariableId, u32>,
}

impl History {
    /// Records what an applied event says about variables.
    pub(crate) fn record(&mut self, event: &Event) {
        match event {
            Event::SharedVariableSet {
                variable,
                owner,
                name,
                value,
            } => {
                self.variables.insert(
                    variable.clone(),
                    (name.clone(), Scope::Shared(owner.clone())),
                );
                if let SharedValue::Secret { version } = value {
                    self.note_version(variable, *version);
                }
            }
            Event::ServiceCreated { service, spec, .. }
            | Event::ServiceSpecChanged { service, spec } => {
                for (name, variable) in &spec.variables {
                    self.variables.insert(
                        variable.id.clone(),
                        (name.clone(), Scope::Service(service.clone())),
                    );
                    if let ServiceVariableValue::OwnSecret { version } = variable.value {
                        self.note_version(&variable.id, version);
                    }
                }
            }
            Event::SecretVersionAdded { variable, version } => {
                self.note_version(variable, *version);
            }
            _ => {}
        }
    }

    fn note_version(&mut self, variable: &VariableId, version: u32) {
        let latest = self.secret_versions.entry(variable.clone()).or_default();
        *latest = (*latest).max(version);
    }

    /// The version a new value of this secret gets: one more than any version it ever had.
    pub(crate) fn next_secret_version(&self, variable: &VariableId) -> u32 {
        self.secret_versions.get(variable).copied().unwrap_or(0) + 1
    }
}

/// What an event is about.
#[derive(Default)]
struct Subject {
    project: Option<ProjectId>,
    environment: Option<EnvironmentId>,
    service: Option<ServiceId>,
    deployment: Option<DeploymentId>,
}

fn failure_text(reason: &FailureReason) -> String {
    match reason {
        FailureReason::ImageResolutionFailed { .. } => "the image could not be pulled".to_owned(),
        FailureReason::ContainerExited { code } => format!("the container exited with code {code}"),
        FailureReason::HealthcheckFailed { .. } => "the healthcheck failed".to_owned(),
        FailureReason::Timeout => "it took more than 5 minutes".to_owned(),
        FailureReason::Replaced => "the service was deleted".to_owned(),
        FailureReason::MissingSecrets { names } => {
            let names = names.iter().map(VarName::as_str).collect::<Vec<_>>();
            format!("missing secrets {}", names.join(", "))
        }
    }
}

impl<S: Storage> Engine<S> {
    /// A page of a project's events, newest first.
    pub(crate) fn list_events(
        &self,
        project_id: &str,
        before: Option<u64>,
        limit: u32,
    ) -> Result<ApiValue, ApiFailure> {
        if !(1..=MAX_EVENTS_PAGE).contains(&limit) {
            return Err(failure(
                FailureKind::Invalid,
                format!("The limit must be between 1 and {MAX_EVENTS_PAGE}."),
            ));
        }
        let project = parse_id(project_id, ProjectId::parse, "project")?;
        if self.world.project(&project).is_none() {
            return Err(not_found("project"));
        }
        let wanted = usize::try_from(limit).unwrap_or(usize::MAX);
        let mut events = Vec::new();
        let mut cursor = before;
        let mut exhausted = false;
        'pages: loop {
            let page = self
                .storage
                .events_page(cursor, limit)
                .map_err(storage_failure)?;
            let full = page.len() == wanted;
            for stored in page {
                cursor = Some(stored.seq);
                let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&stored.bytes) else {
                    continue;
                };
                if let Some(event) = self.activity(&envelope, &project) {
                    events.push(event);
                    if events.len() == wanted {
                        break 'pages;
                    }
                }
            }
            if !full || cursor == Some(1) {
                exhausted = true;
                break;
            }
        }
        let next_before = match (exhausted, events.last()) {
            (false, Some(last)) => Some(last.seq),
            _ => None,
        };
        Ok(ApiValue::Events(EventsPageDto {
            events,
            next_before,
        }))
    }

    fn actor_name(&self, actor: &str) -> String {
        if let Some(admin) = actor.strip_prefix("admin:") {
            return self
                .admin_names
                .iter()
                .find(|(id, _)| id.as_str() == admin)
                .map_or_else(|| "admin".to_owned(), |(_, name)| name.clone());
        }
        if actor.starts_with("agent:") {
            return "agent".to_owned();
        }
        actor.to_owned()
    }

    fn slug(&self, service: &ServiceId) -> String {
        self.world.service(service).map_or_else(
            || "a service".to_owned(),
            |service| service.slug.to_string(),
        )
    }

    fn service_subject(&self, service: &ServiceId) -> Subject {
        let environment = self.world.service(service).map(|s| s.environment.clone());
        Subject {
            project: environment
                .as_ref()
                .and_then(|environment| self.world.project_of_environment(environment))
                .cloned(),
            environment,
            service: Some(service.clone()),
            deployment: None,
        }
    }

    fn deployment_subject(&self, deployment: &DeploymentId) -> (Subject, String) {
        match self.world.deployment(deployment) {
            Some(found) => {
                let mut subject = self.service_subject(&found.service);
                subject.deployment = Some(deployment.clone());
                (subject, self.slug(&found.service))
            }
            None => (Subject::default(), "a service".to_owned()),
        }
    }

    fn owner_subject(&self, owner: &VariableOwner) -> (Subject, String) {
        match owner {
            VariableOwner::Project(project) => (
                Subject {
                    project: Some(project.clone()),
                    ..Subject::default()
                },
                self.world.project(project).map_or_else(
                    || "the project".to_owned(),
                    |p| format!("project {}", p.name),
                ),
            ),
            VariableOwner::Environment(environment) => (
                Subject {
                    project: self.world.project_of_environment(environment).cloned(),
                    environment: Some(environment.clone()),
                    ..Subject::default()
                },
                self.world.environment(environment).map_or_else(
                    || "the environment".to_owned(),
                    |e| format!("environment {}", e.name),
                ),
            ),
        }
    }

    fn variable_subject(&self, variable: &VariableId) -> (Subject, String, String) {
        match self.history.variables.get(variable) {
            Some((name, Scope::Shared(owner))) => {
                let (subject, place) = self.owner_subject(owner);
                (subject, name.to_string(), place)
            }
            Some((name, Scope::Service(service))) => (
                self.service_subject(service),
                name.to_string(),
                self.slug(service),
            ),
            None => (Subject::default(), "a variable".to_owned(), String::new()),
        }
    }

    /// The activity entry of an event of `project`, or `None` for another project's event or
    /// one about no project.
    fn activity(&self, envelope: &EventEnvelope, project: &ProjectId) -> Option<ActivityEventDto> {
        let (subject, summary) = match &envelope.event {
            Event::ProjectCreated { project, name } => (
                Subject {
                    project: Some(project.clone()),
                    ..Subject::default()
                },
                format!("Created project {name}."),
            ),
            Event::EnvironmentCreated {
                environment,
                project,
                name,
            } => (
                Subject {
                    project: Some(project.clone()),
                    environment: Some(environment.clone()),
                    ..Subject::default()
                },
                format!("Created environment {name}."),
            ),
            Event::ServiceCreated {
                service,
                slug,
                spec,
                ..
            } => (
                self.service_subject(service),
                format!("Created service {slug} from {}.", spec.image),
            ),
            Event::ServiceDeleted { service } => (
                self.service_subject(service),
                format!("Deleted service {}.", self.slug(service)),
            ),
            Event::ServiceSpecChanged { service, .. } => (
                self.service_subject(service),
                format!("Changed the settings of {}.", self.slug(service)),
            ),
            Event::ServiceEgressChanged { service, egress } => (
                self.service_subject(service),
                format!(
                    "{} internet access for {}.",
                    if *egress { "Enabled" } else { "Disabled" },
                    self.slug(service)
                ),
            ),
            Event::SharedVariableSet {
                owner, name, value, ..
            } => {
                let (subject, place) = self.owner_subject(owner);
                let what = if value.is_secret() {
                    "secret"
                } else {
                    "variable"
                };
                (subject, format!("Set {what} {name} on {place}."))
            }
            Event::SharedVariableRemoved { variable } => {
                let (subject, name, place) = self.variable_subject(variable);
                (subject, format!("Removed {name} from {place}."))
            }
            Event::SecretVersionAdded { variable, version } => {
                let (subject, name, place) = self.variable_subject(variable);
                (
                    subject,
                    format!("Added version {version} of secret {name} on {place}."),
                )
            }
            Event::DeploymentRequested {
                deployment,
                service,
                reason,
                ..
            } => {
                let mut subject = self.service_subject(service);
                subject.deployment = Some(deployment.clone());
                let why = match reason {
                    DeployReason::Created => "first deployment".to_owned(),
                    DeployReason::Changes => "staged changes".to_owned(),
                    DeployReason::Redeploy => "redeploy".to_owned(),
                    DeployReason::Rollback { .. } => "rollback".to_owned(),
                    DeployReason::SharedVariable { variable } => {
                        let (_, name, _) = self.variable_subject(variable);
                        format!("shared variable {name} changed")
                    }
                };
                (
                    subject,
                    format!("Requested a deployment of {} ({why}).", self.slug(service)),
                )
            }
            Event::RevisionCreated {
                service,
                number,
                spec,
                ..
            } => (
                self.service_subject(service),
                format!(
                    "Created revision {number} of {} with {}.",
                    self.slug(service),
                    spec.digest
                ),
            ),
            Event::DeploymentStarted { deployment, .. } => {
                let (subject, slug) = self.deployment_subject(deployment);
                (subject, format!("Started a container of {slug}."))
            }
            Event::DeploymentHealthy { deployment } => {
                let (subject, slug) = self.deployment_subject(deployment);
                (
                    subject,
                    format!("The new container of {slug} passed its healthcheck."),
                )
            }
            Event::DeploymentSucceeded { deployment } => {
                let (subject, slug) = self.deployment_subject(deployment);
                (subject, format!("The deployment of {slug} is active."))
            }
            Event::DeploymentFailed { deployment, reason } => {
                let (subject, slug) = self.deployment_subject(deployment);
                (
                    subject,
                    format!("The deployment of {slug} failed: {}.", failure_text(reason)),
                )
            }
            Event::SecretRevealed { variable, .. } => {
                let (subject, name, place) = self.variable_subject(variable);
                (subject, format!("Revealed secret {name} of {place}."))
            }
            Event::MachineRegistered { .. }
            | Event::AdminCreated { .. }
            | Event::LoginSucceeded { .. }
            | Event::LoginFailed
            | Event::SessionClosed { .. } => return None,
        };
        if subject.project.as_ref() != Some(project) {
            return None;
        }
        Some(ActivityEventDto {
            seq: envelope.seq,
            at: envelope.at.to_rfc3339(),
            kind: envelope.event.type_name().to_owned(),
            actor: self.actor_name(&envelope.actor),
            environment_id: subject.environment.map(|id| id.to_string()),
            service_id: subject.service.map(|id| id.to_string()),
            deployment_id: subject.deployment.map(|id| id.to_string()),
            summary,
        })
    }

    /// The build or deploy log of a deployment, derived from its events.
    pub(crate) fn deployment_logs(
        &self,
        deployment_id: &str,
        stream: LogStream,
    ) -> Result<ApiValue, ApiFailure> {
        let deployment = parse_id(deployment_id, DeploymentId::parse, "deployment")?;
        let deployment = self
            .world
            .deployment(&deployment)
            .ok_or_else(|| not_found("deployment"))?;
        Ok(ApiValue::LogLines(self.log_lines(deployment, stream)))
    }

    fn log_lines(&self, deployment: &Deployment, stream: LogStream) -> Vec<LogLineDto> {
        let revision = deployment
            .revision
            .as_ref()
            .and_then(|revision| self.world.revision(revision));
        let mut lines: Vec<(Timestamp, LogLevel, String)> = Vec::new();
        let failed = match &deployment.phase {
            DeploymentPhase::Failed(reason) => deployment.finished_at.map(|at| (at, reason)),
            _ => None,
        };
        match stream {
            LogStream::Build => {
                match &deployment.plan {
                    RevisionPlan::Resolve { image } => {
                        lines.push((
                            deployment.created_at,
                            LogLevel::Info,
                            format!("Resolving image {image}"),
                        ));
                        if let Some(revision) = revision {
                            lines.push((
                                revision.created_at,
                                LogLevel::Info,
                                format!("Resolved {}", revision.spec.digest),
                            ));
                        }
                    }
                    RevisionPlan::Reuse { digest, .. } => lines.push((
                        deployment.created_at,
                        LogLevel::Info,
                        format!("Reusing image {digest}"),
                    )),
                    RevisionPlan::Existing { .. } => {
                        let number = revision.map_or(0, |revision| revision.number);
                        lines.push((
                            deployment.created_at,
                            LogLevel::Info,
                            format!("Reusing revision {number}"),
                        ));
                    }
                }
                if let Some((at, FailureReason::ImageResolutionFailed { message })) = failed {
                    lines.push((
                        at,
                        LogLevel::Error,
                        format!("Image resolution failed: {message}"),
                    ));
                }
            }
            LogStream::Deploy => {
                if let Some(at) = deployment.started_at {
                    lines.push((at, LogLevel::Info, "Starting container".to_owned()));
                }
                if let Some(at) = deployment.healthy_at {
                    lines.push((at, LogLevel::Info, "Healthcheck passed".to_owned()));
                }
                if matches!(
                    deployment.phase,
                    DeploymentPhase::Active | DeploymentPhase::Superseded
                ) && let Some(at) = deployment.finished_at
                {
                    lines.push((at, LogLevel::Info, "Deployment active".to_owned()));
                }
                if let Some((at, reason)) = failed {
                    let message = match reason {
                        FailureReason::ImageResolutionFailed { message } => {
                            format!("Image resolution failed: {message}")
                        }
                        FailureReason::HealthcheckFailed { reason } => {
                            format!("Healthcheck failed: {reason}")
                        }
                        other => format!("Deployment failed: {}", failure_text(other)),
                    };
                    lines.push((at, LogLevel::Error, message));
                }
            }
        }
        let stream_name = match stream {
            LogStream::Build => "build",
            LogStream::Deploy => "deploy",
        };
        lines
            .into_iter()
            .enumerate()
            .map(|(index, (at, level, message))| LogLineDto {
                id: format!("{}-{stream_name}-{index}", deployment.id),
                timestamp: at.to_rfc3339(),
                service_id: deployment.service.to_string(),
                level,
                message,
            })
            .collect()
    }
}
