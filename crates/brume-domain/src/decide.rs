//! Decides which events a command produces, and resolves variables and secrets.

use std::collections::{BTreeMap, BTreeSet};

use crate::command::{AuditEvent, Command, NewService, ServiceChanges, SpecChange};
use crate::event::{Event, RevisionPlan};
use crate::evolve::apply;
use crate::ids::{
    AdminId, DeploymentId, EnvironmentId, ProjectId, RevisionId, ServiceId, VariableId,
};
use crate::model::{
    DeployReason, Deployment, DeploymentPhase, FailureReason, Level, Project, ResolvedSecret,
    Revision, RevisionSpec, RevisionValue, RevisionVariable, Service, ServiceSpec, ServiceVariable,
    ServiceVariableValue, SharedValue, VariableOwner, World,
};
use crate::names::{Slug, VarName};
use crate::rejection::Rejection;
use crate::time::Timestamp;

/// Decides the events of a command, or why it is refused. Pure: the same world and command
/// always give the same answer.
pub fn decide(world: &World, command: &Command) -> Result<Vec<Event>, Rejection> {
    let mut decision = Decision::new(world);
    match command {
        Command::Bootstrap {
            machine,
            admin,
            identity,
        } => {
            if world.machine.is_some() || !world.admins.is_empty() {
                return Err(Rejection::conflict(
                    "This server is already set up.".to_owned(),
                ));
            }
            decision.push(Event::MachineRegistered {
                machine: machine.clone(),
            });
            decision.push(Event::AdminCreated {
                admin: admin.clone(),
                identity: identity.clone(),
            });
        }
        Command::CreateProject {
            project,
            name,
            production,
            first_service,
        } => {
            if world.projects.contains_key(project) || world.environments.contains_key(production) {
                return Err(Rejection::conflict(
                    "This project already exists.".to_owned(),
                ));
            }
            decision.push(Event::ProjectCreated {
                project: project.clone(),
                name: name.clone(),
            });
            decision.push(Event::EnvironmentCreated {
                environment: production.clone(),
                project: project.clone(),
                name: Slug::parse("production").expect("production is a valid slug"),
            });
            if let Some(service) = first_service {
                if &service.environment != production {
                    return Err(Rejection::invalid(
                        "The first service must be in the production environment.".to_owned(),
                    ));
                }
                decision.create_service(service)?;
            }
        }
        Command::CreateEnvironment {
            environment,
            project,
            name,
        } => {
            require_project(world, project)?;
            if world.environments.contains_key(environment) {
                return Err(Rejection::conflict(
                    "This environment already exists.".to_owned(),
                ));
            }
            if world
                .environments_of(project)
                .any(|existing| &existing.name == name)
            {
                return Err(Rejection::conflict(format!(
                    "An environment named {name} already exists in this project."
                )));
            }
            decision.push(Event::EnvironmentCreated {
                environment: environment.clone(),
                project: project.clone(),
                name: name.clone(),
            });
        }
        Command::CreateService(service) => decision.create_service(service)?,
        Command::DeleteService { service } => {
            let target = require_live_service(world, service)?;
            if let Some(deployment) = world.in_progress_deployment(&target.id) {
                decision.push(Event::DeploymentFailed {
                    deployment: deployment.id.clone(),
                    reason: FailureReason::Replaced,
                });
            }
            decision.push(Event::ServiceDeleted {
                service: service.clone(),
            });
        }
        Command::DeployChanges {
            environment,
            services,
            author,
        } => decision.deploy_changes(environment, services, author)?,
        Command::Redeploy {
            service,
            deployment,
            author,
        } => {
            let target = require_live_service(world, service)?;
            check_no_deployment_in_progress(world, target)?;
            let Some(revision) = world.active_revision(service) else {
                return Err(Rejection::conflict(format!(
                    "{} has no active deployment to redeploy.",
                    target.slug
                )));
            };
            check_secrets_resolve(world, service, revision)?;
            decision.request_existing(
                service,
                deployment,
                &revision.id,
                DeployReason::Redeploy,
                author,
            );
        }
        Command::Rollback {
            target,
            deployment,
            author,
        } => decision.rollback(target, deployment, author)?,
        Command::SetSharedVariable {
            variable,
            owner,
            name,
            value,
            deployments,
            author,
        } => {
            let project = owner_project(world, owner)?;
            if let Some(existing) = world.shared_variables.get(variable)
                && (&existing.owner != owner || &existing.name != name)
            {
                return Err(Rejection::invalid(
                    "This variable id belongs to another variable.".to_owned(),
                ));
            }
            if let Some(existing) = world.shared_variable_named(owner, name)
                && &existing.id != variable
            {
                return Err(Rejection::invalid(format!(
                    "{name} already exists here with another id."
                )));
            }
            if matches!(value, SharedValue::Secret { version: 0 }) {
                return Err(Rejection::invalid(
                    "A secret version starts at 1.".to_owned(),
                ));
            }
            decision.push(Event::SharedVariableSet {
                variable: variable.clone(),
                owner: owner.clone(),
                name: name.clone(),
                value: value.clone(),
            });
            check_names(&decision.world, &project)?;
            let impacted = match value {
                SharedValue::Plain { .. } => impacted_services(&decision.world, owner, name),
                SharedValue::Secret { .. } => Vec::new(),
            };
            decision.redeploy_impacted(&impacted, deployments, variable, author)?;
        }
        Command::RemoveSharedVariable {
            variable,
            deployments,
            author,
        } => {
            let Some(shared) = world.shared_variables.get(variable) else {
                return Err(Rejection::not_found(
                    "This variable does not exist.".to_owned(),
                ));
            };
            let impacted = match shared.value {
                SharedValue::Plain { .. } => impacted_services(world, &shared.owner, &shared.name),
                SharedValue::Secret { .. } => {
                    let users = secret_references(world, &shared.owner, &shared.name);
                    if !users.is_empty() {
                        let slugs = users
                            .iter()
                            .filter_map(|service| world.service(service))
                            .map(|service| service.slug.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Err(Rejection::conflict(format!(
                            "{} is still used by: {slugs}.",
                            shared.name
                        )));
                    }
                    Vec::new()
                }
            };
            decision.push(Event::SharedVariableRemoved {
                variable: variable.clone(),
            });
            decision.redeploy_impacted(&impacted, deployments, variable, author)?;
        }
        Command::ImageResolved {
            deployment,
            revision,
            digest,
            volumes,
        } => {
            let target = require_phase(world, deployment, Phase::Resolving)?;
            let service = require_live_service(world, &target.service)?;
            if world.revisions.contains_key(revision) {
                return Err(Rejection::conflict(
                    "This revision already exists.".to_owned(),
                ));
            }
            let number = world
                .revisions_of(&service.id)
                .map(|revision| revision.number)
                .max()
                .unwrap_or(0)
                + 1;
            let spec = RevisionSpec {
                image: service.spec.image.clone(),
                digest: digest.clone(),
                volumes: volumes.clone(),
                start_command: service.spec.start_command.clone(),
                port: service.spec.port,
                healthcheck_path: service.spec.healthcheck_path.clone(),
                cpu: service.spec.cpu,
                memory: service.spec.memory,
                variables: resolve_revision_variables(world, &service.id),
            };
            decision.push(Event::RevisionCreated {
                revision: revision.clone(),
                service: service.id.clone(),
                number,
                spec,
            });
            decision.start(deployment, revision);
        }
        Command::ImageResolutionFailed {
            deployment,
            message,
        } => {
            require_phase(world, deployment, Phase::Resolving)?;
            decision.push(Event::DeploymentFailed {
                deployment: deployment.clone(),
                reason: FailureReason::ImageResolutionFailed {
                    message: message.clone(),
                },
            });
        }
        Command::DeploymentHealthy { deployment } => {
            require_phase(world, deployment, Phase::Starting)?;
            decision.push(Event::DeploymentHealthy {
                deployment: deployment.clone(),
            });
        }
        Command::DeploymentPromoted { deployment } => {
            require_phase(world, deployment, Phase::Promoting)?;
            decision.push(Event::DeploymentSucceeded {
                deployment: deployment.clone(),
            });
        }
        Command::DeploymentFailed { deployment, reason } => {
            require_phase(world, deployment, Phase::InProgress)?;
            decision.push(Event::DeploymentFailed {
                deployment: deployment.clone(),
                reason: reason.clone(),
            });
        }
        Command::RecordAudit(audit) => decision.push(match audit {
            AuditEvent::SecretRevealed { variable, admin } => Event::SecretRevealed {
                variable: variable.clone(),
                admin: admin.clone(),
            },
            AuditEvent::LoginSucceeded { admin } => Event::LoginSucceeded {
                admin: admin.clone(),
            },
            AuditEvent::LoginFailed => Event::LoginFailed,
            AuditEvent::SessionClosed { admin } => Event::SessionClosed {
                admin: admin.clone(),
            },
        }),
    }
    Ok(decision.events)
}

/// The variables of the revision that a deployment of `service` would create now: the
/// service's, then the environment's, then the project's, sorted by name. Only plain variables
/// are inherited; a shared secret is kept, by name, only if the service references it.
pub fn resolve_revision_variables(world: &World, service: &ServiceId) -> Vec<RevisionVariable> {
    let Some(service) = world.services.get(service) else {
        return Vec::new();
    };
    let Some(project) = world.project_of_environment(&service.environment) else {
        return Vec::new();
    };
    let mut resolved = BTreeMap::new();
    let inherited = [
        (VariableOwner::Project(project.clone()), Level::Project),
        (
            VariableOwner::Environment(service.environment.clone()),
            Level::Environment,
        ),
    ];
    for (owner, origin) in inherited {
        for shared in world.shared_variables.values() {
            if let (true, SharedValue::Plain { kind, blob }) =
                (shared.owner == owner, &shared.value)
            {
                resolved.insert(
                    shared.name.clone(),
                    RevisionVariable {
                        name: shared.name.clone(),
                        origin,
                        value: RevisionValue::Plain {
                            kind: *kind,
                            blob: blob.clone(),
                            variable: shared.id.clone(),
                        },
                    },
                );
            }
        }
    }
    for (name, variable) in &service.spec.variables {
        let (origin, value) = match &variable.value {
            ServiceVariableValue::Plain { kind, blob } => (
                Level::Service,
                RevisionValue::Plain {
                    kind: *kind,
                    blob: blob.clone(),
                    variable: variable.id.clone(),
                },
            ),
            ServiceVariableValue::OwnSecret { .. } => (
                Level::Service,
                RevisionValue::Secret {
                    service_variable: Some(variable.id.clone()),
                    reference: None,
                },
            ),
            ServiceVariableValue::SharedSecret => (
                shared_secret(world, service, name).map_or(Level::Service, |(level, _)| level),
                RevisionValue::Secret {
                    service_variable: None,
                    reference: Some(variable.id.clone()),
                },
            ),
        };
        resolved.insert(
            name.clone(),
            RevisionVariable {
                name: name.clone(),
                origin,
                value,
            },
        );
    }
    resolved.into_values().collect()
}

/// The current version of each secret, by name: the service's own secret, else the shared
/// secret of the environment, else the project's. `Err` holds the names that resolve nowhere.
pub fn resolve_secrets(
    world: &World,
    service: &ServiceId,
    names: &[VarName],
) -> Result<Vec<ResolvedSecret>, Vec<VarName>> {
    let Some(service) = world.services.get(service) else {
        return Err(names.to_vec());
    };
    let mut resolved = Vec::new();
    let mut missing = Vec::new();
    for name in names {
        let found = match service
            .spec
            .variables
            .get(name)
            .map(|variable| &variable.value)
        {
            Some(ServiceVariableValue::OwnSecret { version }) => {
                let id = &service.spec.variables[name].id;
                Some((id.clone(), *version))
            }
            Some(ServiceVariableValue::Plain { .. }) => None,
            Some(ServiceVariableValue::SharedSecret) | None => {
                shared_secret(world, service, name).map(|(_, found)| found)
            }
        };
        match found {
            Some((variable, version)) => resolved.push(ResolvedSecret {
                name: name.clone(),
                variable,
                version,
            }),
            None => missing.push(name.clone()),
        }
    }
    if missing.is_empty() {
        Ok(resolved)
    } else {
        Err(missing)
    }
}

/// The live services that inherit the shared variable `name` of `owner` and do not define it
/// at a more precise level.
pub fn impacted_services(world: &World, owner: &VariableOwner, name: &VarName) -> Vec<ServiceId> {
    let services: Vec<&Service> = match owner {
        VariableOwner::Project(project) => world
            .services_of_project(project)
            .filter(|service| {
                world
                    .shared_variable_named(
                        &VariableOwner::Environment(service.environment.clone()),
                        name,
                    )
                    .is_none()
            })
            .collect(),
        VariableOwner::Environment(environment) => world.services_in(environment).collect(),
    };
    services
        .into_iter()
        .filter(|service| !service.spec.variables.contains_key(name))
        .map(|service| service.id.clone())
        .collect()
}

/// The live services whose active deployment uses the shared secret `name` of `owner`.
pub fn secret_references(world: &World, owner: &VariableOwner, name: &VarName) -> Vec<ServiceId> {
    let Some(shared) = world.shared_variable_named(owner, name) else {
        return Vec::new();
    };
    world
        .services
        .values()
        .filter(|service| !service.deleted)
        .filter(|service| {
            world.active_deployment(&service.id).is_some_and(|active| {
                active
                    .secrets
                    .iter()
                    .any(|secret| secret.variable == shared.id)
            })
        })
        .map(|service| service.id.clone())
        .collect()
}

/// The shared secret that a reference from `service` to `name` resolves to, and its level.
fn shared_secret(
    world: &World,
    service: &Service,
    name: &VarName,
) -> Option<(Level, (VariableId, u32))> {
    let project = world.project_of_environment(&service.environment)?;
    let levels = [
        (
            VariableOwner::Environment(service.environment.clone()),
            Level::Environment,
        ),
        (VariableOwner::Project(project.clone()), Level::Project),
    ];
    levels.into_iter().find_map(|(owner, level)| {
        match world
            .shared_variable_named(&owner, name)
            .map(|shared| (shared, &shared.value))
        {
            Some((shared, SharedValue::Secret { version })) => {
                Some((level, (shared.id.clone(), *version)))
            }
            _ => None,
        }
    })
}

/// The events of a decision, applied as they come to a scratch copy of the world, so that a
/// later step sees the earlier ones.
struct Decision {
    world: World,
    events: Vec<Event>,
}

/// The instant of the scratch world's events. Decisions never read it.
const SCRATCH_TIME: Timestamp = Timestamp::from_millis(0);

impl Decision {
    fn new(world: &World) -> Self {
        Self {
            world: world.clone(),
            events: Vec::new(),
        }
    }

    fn push(&mut self, event: Event) {
        apply(&mut self.world, &event, SCRATCH_TIME);
        self.events.push(event);
    }

    fn create_service(&mut self, new: &NewService) -> Result<(), Rejection> {
        if !self.world.environments.contains_key(&new.environment) {
            return Err(Rejection::not_found(
                "This environment does not exist.".to_owned(),
            ));
        }
        if self.world.services.contains_key(&new.service)
            || self.world.deployments.contains_key(&new.deployment)
        {
            return Err(Rejection::conflict(
                "This service already exists.".to_owned(),
            ));
        }
        if self
            .world
            .service_by_slug(&new.environment, &new.slug)
            .is_some()
        {
            return Err(Rejection::conflict(format!(
                "A service named {} already exists in this environment.",
                new.slug
            )));
        }
        self.push(Event::ServiceCreated {
            service: new.service.clone(),
            environment: new.environment.clone(),
            name: new.name.clone(),
            slug: new.slug.clone(),
            spec: ServiceSpec::new(new.image.clone()),
        });
        self.push(Event::DeploymentRequested {
            deployment: new.deployment.clone(),
            service: new.service.clone(),
            revision: RevisionPlan::Resolve {
                image: new.image.clone(),
            },
            reason: DeployReason::Created,
            author: new.author.clone(),
        });
        Ok(())
    }

    fn deploy_changes(
        &mut self,
        environment: &EnvironmentId,
        services: &[ServiceChanges],
        author: &AdminId,
    ) -> Result<(), Rejection> {
        let Some(project) = self.world.project_of_environment(environment).cloned() else {
            return Err(Rejection::not_found(
                "This environment does not exist.".to_owned(),
            ));
        };
        if services.is_empty() {
            return Err(Rejection::invalid(
                "There are no changes to deploy.".to_owned(),
            ));
        }
        for changes in services {
            let service = require_live_service(&self.world, &changes.service)?.clone();
            if &service.environment != environment {
                return Err(Rejection::invalid(format!(
                    "{} is not in this environment.",
                    service.slug
                )));
            }
            check_no_deployment_in_progress(&self.world, &service)?;
            let mut spec = service.spec.clone();
            let mut egress = service.egress;
            let mut versions = Vec::new();
            for change in &changes.changes {
                match change {
                    SpecChange::Image(image) => spec.image = image.clone(),
                    SpecChange::StartCommand(command) => spec.start_command = command.clone(),
                    SpecChange::Port(port) => spec.port = *port,
                    SpecChange::HealthcheckPath(path) => spec.healthcheck_path = path.clone(),
                    SpecChange::Cpu(cpu) => spec.cpu = *cpu,
                    SpecChange::Memory(memory) => spec.memory = *memory,
                    SpecChange::Egress(enabled) => egress = *enabled,
                    SpecChange::PlainVariable {
                        name,
                        variable,
                        kind,
                        blob,
                    } => {
                        spec.variables.insert(
                            name.clone(),
                            ServiceVariable {
                                id: variable.clone(),
                                value: ServiceVariableValue::Plain {
                                    kind: *kind,
                                    blob: blob.clone(),
                                },
                            },
                        );
                    }
                    SpecChange::OwnSecret {
                        name,
                        variable,
                        version,
                    } => {
                        if *version == 0 {
                            return Err(Rejection::invalid(
                                "A secret version starts at 1.".to_owned(),
                            ));
                        }
                        spec.variables.insert(
                            name.clone(),
                            ServiceVariable {
                                id: variable.clone(),
                                value: ServiceVariableValue::OwnSecret { version: *version },
                            },
                        );
                        versions.push((variable.clone(), *version));
                    }
                    SpecChange::SharedSecretReference { name, variable } => {
                        spec.variables.insert(
                            name.clone(),
                            ServiceVariable {
                                id: variable.clone(),
                                value: ServiceVariableValue::SharedSecret,
                            },
                        );
                    }
                    SpecChange::RemoveVariable { name } => {
                        if spec.variables.remove(name).is_none() {
                            return Err(Rejection::not_found(format!(
                                "{name} is not a variable of {}.",
                                service.slug
                            )));
                        }
                    }
                }
            }
            if spec != service.spec {
                self.push(Event::ServiceSpecChanged {
                    service: service.id.clone(),
                    spec,
                });
            }
            for (variable, version) in versions {
                self.push(Event::SecretVersionAdded { variable, version });
            }
            if egress != service.egress {
                self.push(Event::ServiceEgressChanged {
                    service: service.id.clone(),
                    egress,
                });
            }
            check_names(&self.world, &project)?;
            self.request(
                &service.id,
                &changes.deployment,
                DeployReason::Changes,
                author,
            );
        }
        Ok(())
    }

    fn rollback(
        &mut self,
        target: &DeploymentId,
        deployment: &DeploymentId,
        author: &AdminId,
    ) -> Result<(), Rejection> {
        let Some(earlier) = self.world.deployments.get(target) else {
            return Err(Rejection::not_found(
                "This deployment does not exist.".to_owned(),
            ));
        };
        let Some(revision) = earlier
            .revision
            .as_ref()
            .and_then(|revision| self.world.revisions.get(revision))
            .cloned()
        else {
            return Err(Rejection::conflict(
                "This deployment has no revision to roll back to.".to_owned(),
            ));
        };
        let service = require_live_service(&self.world, &earlier.service)?.clone();
        check_no_deployment_in_progress(&self.world, &service)?;
        let project = self
            .world
            .project_of_environment(&service.environment)
            .cloned()
            .expect("a live service has a project");
        let spec = restored_spec(&service, &revision)?;
        self.push(Event::ServiceSpecChanged {
            service: service.id.clone(),
            spec,
        });
        check_names(&self.world, &project)?;
        check_secrets_resolve(&self.world, &service.id, &revision)?;
        self.request_existing(
            &service.id,
            deployment,
            &revision.id,
            DeployReason::Rollback {
                target: target.clone(),
            },
            author,
        );
        Ok(())
    }

    /// Redeploys exactly the impacted services, each with the deployment id the engine drew.
    fn redeploy_impacted(
        &mut self,
        impacted: &[ServiceId],
        deployments: &[(ServiceId, DeploymentId)],
        variable: &VariableId,
        author: &AdminId,
    ) -> Result<(), Rejection> {
        let expected = impacted.iter().collect::<BTreeSet<_>>();
        let given = deployments
            .iter()
            .map(|(service, _)| service)
            .collect::<BTreeSet<_>>();
        if expected != given || given.len() != deployments.len() {
            return Err(Rejection::conflict(
                "The services that this variable reaches changed. Review them and try again."
                    .to_owned(),
            ));
        }
        for (service, deployment) in deployments {
            let target = require_live_service(&self.world, service)?.clone();
            check_no_deployment_in_progress(&self.world, &target)?;
            self.request(
                service,
                deployment,
                DeployReason::SharedVariable {
                    variable: variable.clone(),
                },
                author,
            );
        }
        Ok(())
    }

    /// Requests a deployment of the service as it is now: the active revision again if
    /// nothing in it would change, its digest with new settings, or a new image.
    fn request(
        &mut self,
        service: &ServiceId,
        deployment: &DeploymentId,
        reason: DeployReason,
        author: &AdminId,
    ) {
        let current = &self.world.services[service];
        let plan = match self.world.active_revision(service) {
            Some(active) if active.spec.image == current.spec.image => {
                let spec = &current.spec;
                let unchanged = active.spec.start_command == spec.start_command
                    && active.spec.port == spec.port
                    && active.spec.healthcheck_path == spec.healthcheck_path
                    && active.spec.cpu == spec.cpu
                    && active.spec.memory == spec.memory
                    && active.spec.variables == resolve_revision_variables(&self.world, service);
                if unchanged {
                    let revision = active.id.clone();
                    self.request_existing(service, deployment, &revision, reason, author);
                    return;
                }
                RevisionPlan::Reuse {
                    digest: active.spec.digest.clone(),
                    volumes: active.spec.volumes.clone(),
                }
            }
            _ => RevisionPlan::Resolve {
                image: current.spec.image.clone(),
            },
        };
        self.push(Event::DeploymentRequested {
            deployment: deployment.clone(),
            service: service.clone(),
            revision: plan,
            reason,
            author: author.clone(),
        });
    }

    /// Requests and starts a deployment of an existing revision.
    fn request_existing(
        &mut self,
        service: &ServiceId,
        deployment: &DeploymentId,
        revision: &RevisionId,
        reason: DeployReason,
        author: &AdminId,
    ) {
        self.push(Event::DeploymentRequested {
            deployment: deployment.clone(),
            service: service.clone(),
            revision: RevisionPlan::Existing {
                revision: revision.clone(),
            },
            reason,
            author: author.clone(),
        });
        self.start(deployment, revision);
    }

    /// Starts a deployment with the secrets resolved now, or fails it if one is missing.
    fn start(&mut self, deployment: &DeploymentId, revision: &RevisionId) {
        let service = self.world.deployments[deployment].service.clone();
        let names = secret_names(&self.world.revisions[revision]);
        match resolve_secrets(&self.world, &service, &names) {
            Ok(secrets) => self.push(Event::DeploymentStarted {
                deployment: deployment.clone(),
                revision: revision.clone(),
                secrets,
            }),
            Err(names) => self.push(Event::DeploymentFailed {
                deployment: deployment.clone(),
                reason: FailureReason::MissingSecrets { names },
            }),
        }
    }
}

/// The service's spec put back to a revision: its settings and its service-level variables.
/// The service's own secrets keep their current version; one that no longer exists is missing.
fn restored_spec(service: &Service, revision: &Revision) -> Result<ServiceSpec, Rejection> {
    let mut variables = BTreeMap::new();
    let mut missing = Vec::new();
    for variable in &revision.spec.variables {
        let current = service.spec.variables.get(&variable.name);
        let restored = match (&variable.value, variable.origin) {
            (
                RevisionValue::Plain {
                    kind,
                    blob,
                    variable,
                },
                Level::Service,
            ) => Some(ServiceVariable {
                id: variable.clone(),
                value: ServiceVariableValue::Plain {
                    kind: *kind,
                    blob: blob.clone(),
                },
            }),
            (RevisionValue::Plain { .. }, Level::Environment | Level::Project) => None,
            (
                RevisionValue::Secret {
                    service_variable: Some(_),
                    ..
                },
                _,
            ) => match current {
                Some(
                    own @ ServiceVariable {
                        value: ServiceVariableValue::OwnSecret { .. },
                        ..
                    },
                ) => Some(own.clone()),
                _ => {
                    missing.push(variable.name.clone());
                    None
                }
            },
            (
                RevisionValue::Secret {
                    service_variable: None,
                    reference,
                },
                _,
            ) => {
                let id = match (current, reference) {
                    (
                        Some(ServiceVariable {
                            id,
                            value: ServiceVariableValue::SharedSecret,
                        }),
                        _,
                    ) => Some(id.clone()),
                    (_, Some(reference)) => Some(reference.clone()),
                    (_, None) => None,
                };
                match id {
                    Some(id) => Some(ServiceVariable {
                        id,
                        value: ServiceVariableValue::SharedSecret,
                    }),
                    None => {
                        missing.push(variable.name.clone());
                        None
                    }
                }
            }
        };
        if let Some(restored) = restored {
            variables.insert(variable.name.clone(), restored);
        }
    }
    if !missing.is_empty() {
        return Err(missing_secrets(&missing));
    }
    Ok(ServiceSpec {
        image: revision.spec.image.clone(),
        start_command: revision.spec.start_command.clone(),
        port: revision.spec.port,
        healthcheck_path: revision.spec.healthcheck_path.clone(),
        cpu: revision.spec.cpu,
        memory: revision.spec.memory,
        variables,
    })
}

fn secret_names(revision: &Revision) -> Vec<VarName> {
    revision
        .spec
        .variables
        .iter()
        .filter(|variable| matches!(variable.value, RevisionValue::Secret { .. }))
        .map(|variable| variable.name.clone())
        .collect()
}

fn check_secrets_resolve(
    world: &World,
    service: &ServiceId,
    revision: &Revision,
) -> Result<(), Rejection> {
    resolve_secrets(world, service, &secret_names(revision))
        .map(|_| ())
        .map_err(|missing| missing_secrets(&missing))
}

fn missing_secrets(names: &[VarName]) -> Rejection {
    let names = names
        .iter()
        .map(VarName::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    Rejection::conflict(format!("These secrets are missing: {names}."))
}

/// Refuses a project where a name is both a plain variable and a secret, at any level.
fn check_names(world: &World, project: &ProjectId) -> Result<(), Rejection> {
    let mut classes: BTreeMap<&VarName, BTreeSet<bool>> = BTreeMap::new();
    for shared in world.shared_variables.values() {
        let in_project = match &shared.owner {
            VariableOwner::Project(owner) => owner == project,
            VariableOwner::Environment(environment) => {
                world.project_of_environment(environment) == Some(project)
            }
        };
        if in_project {
            classes
                .entry(&shared.name)
                .or_default()
                .insert(shared.value.is_secret());
        }
    }
    for service in world.services_of_project(project) {
        for (name, variable) in &service.spec.variables {
            classes
                .entry(name)
                .or_default()
                .insert(variable.value.is_secret());
        }
    }
    match classes.into_iter().find(|(_, kinds)| kinds.len() > 1) {
        Some((name, _)) => Err(Rejection::conflict(format!(
            "{name} cannot be both a variable and a secret in the same project."
        ))),
        None => Ok(()),
    }
}

fn require_project<'a>(world: &'a World, project: &ProjectId) -> Result<&'a Project, Rejection> {
    world
        .projects
        .get(project)
        .ok_or_else(|| Rejection::not_found("This project does not exist.".to_owned()))
}

fn require_live_service<'a>(
    world: &'a World,
    service: &ServiceId,
) -> Result<&'a Service, Rejection> {
    world
        .live_service(service)
        .ok_or_else(|| Rejection::not_found("This service does not exist.".to_owned()))
}

fn owner_project(world: &World, owner: &VariableOwner) -> Result<ProjectId, Rejection> {
    match owner {
        VariableOwner::Project(project) => require_project(world, project).map(|p| p.id.clone()),
        VariableOwner::Environment(environment) => world
            .project_of_environment(environment)
            .cloned()
            .ok_or_else(|| Rejection::not_found("This environment does not exist.".to_owned())),
    }
}

fn check_no_deployment_in_progress(world: &World, service: &Service) -> Result<(), Rejection> {
    match world.in_progress_deployment(&service.id) {
        Some(_) => Err(Rejection::conflict(format!(
            "A deployment is already in progress for {}.",
            service.slug
        ))),
        None => Ok(()),
    }
}

/// The phases a fact about a deployment requires.
enum Phase {
    Resolving,
    Starting,
    Promoting,
    InProgress,
}

fn require_phase<'a>(
    world: &'a World,
    deployment: &DeploymentId,
    phase: Phase,
) -> Result<&'a Deployment, Rejection> {
    let Some(target) = world.deployments.get(deployment) else {
        return Err(Rejection::not_found(
            "This deployment does not exist.".to_owned(),
        ));
    };
    let matches = match phase {
        Phase::Resolving => target.phase == DeploymentPhase::Resolving,
        Phase::Starting => target.phase == DeploymentPhase::Starting,
        Phase::Promoting => target.phase == DeploymentPhase::Promoting,
        Phase::InProgress => target.phase.is_in_progress(),
    };
    if !matches {
        return Err(Rejection::conflict(
            "This deployment is no longer at that step.".to_owned(),
        ));
    }
    Ok(target)
}
