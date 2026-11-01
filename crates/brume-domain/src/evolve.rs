//! Applies the facts of the log to the `World`.

use crate::event::{Event, EventEnvelope, RevisionPlan};
use crate::ids::{DeploymentId, ProjectId, ServiceId};
use crate::model::{
    Deployment, DeploymentPhase, Environment, Ingress, Project, Revision, Service, SharedVariable,
    VariableOwner, World,
};
use crate::time::Timestamp;

/// Applies an event to the `World`. An event is a fact: one that does not apply is a bug, so
/// this panics.
pub fn evolve(world: &mut World, envelope: &EventEnvelope) {
    apply(world, &envelope.event, envelope.at);
}

/// Applies an event recorded at `at`.
pub(crate) fn apply(world: &mut World, event: &Event, at: Timestamp) {
    match event {
        Event::MachineRegistered { machine } => world.machine = Some(machine.clone()),
        Event::AdminCreated { admin, identity } => {
            world.admins.insert(admin.clone(), identity.clone());
        }
        Event::ProjectCreated { project, name } => {
            world.projects.insert(
                project.clone(),
                Project {
                    id: project.clone(),
                    name: name.clone(),
                    description: String::new(),
                    created_at: at,
                    updated_at: at,
                },
            );
        }
        Event::EnvironmentCreated {
            environment,
            project,
            name,
        } => {
            assert!(
                world.projects.contains_key(project),
                "EnvironmentCreated for an unknown project"
            );
            world.environments.insert(
                environment.clone(),
                Environment {
                    id: environment.clone(),
                    project: project.clone(),
                    name: name.clone(),
                },
            );
            touch_project(world, project, at);
        }
        Event::ServiceCreated {
            service,
            environment,
            name,
            slug,
            spec,
        } => {
            let project = world
                .project_of_environment(environment)
                .expect("ServiceCreated for an unknown environment")
                .clone();
            world.services.insert(
                service.clone(),
                Service {
                    id: service.clone(),
                    environment: environment.clone(),
                    name: name.clone(),
                    slug: slug.clone(),
                    egress: false,
                    domain: None,
                    volumes: Vec::new(),
                    spec: spec.clone(),
                    deleted: false,
                },
            );
            touch_project(world, &project, at);
        }
        Event::ServiceDeleted { service } => {
            service_mut(world, service).deleted = true;
            touch_service_project(world, service, at);
        }
        Event::ServiceSpecChanged { service, spec } => {
            service_mut(world, service).spec = spec.clone();
            touch_service_project(world, service, at);
        }
        Event::ServiceEgressChanged { service, egress } => {
            service_mut(world, service).egress = *egress;
            touch_service_project(world, service, at);
        }
        Event::ServiceDomainChanged { service, domain } => {
            service_mut(world, service).domain = domain.clone();
            touch_service_project(world, service, at);
        }
        Event::SharedVariableSet {
            variable,
            owner,
            name,
            value,
        } => {
            let owner_exists = match owner {
                VariableOwner::Project(project) => world.projects.contains_key(project),
                VariableOwner::Environment(environment) => {
                    world.environments.contains_key(environment)
                }
            };
            assert!(owner_exists, "SharedVariableSet for an unknown owner");
            world.shared_variables.insert(
                variable.clone(),
                SharedVariable {
                    id: variable.clone(),
                    owner: owner.clone(),
                    name: name.clone(),
                    value: value.clone(),
                },
            );
            touch_owner_project(world, owner, at);
        }
        Event::SharedVariableRemoved { variable } => {
            let removed = world
                .shared_variables
                .remove(variable)
                .expect("SharedVariableRemoved for an unknown variable");
            touch_owner_project(world, &removed.owner, at);
        }
        Event::SecretVersionAdded { .. } => {
            // The version is in the `ServiceSpecChanged` of the same decision; this event
            // records it on the variable's own stream.
        }
        Event::DeploymentRequested {
            deployment,
            service,
            revision,
            reason,
            author,
        } => {
            assert!(
                world.services.contains_key(service),
                "DeploymentRequested for an unknown service"
            );
            assert!(
                !world.deployments.contains_key(deployment),
                "DeploymentRequested for an existing deployment"
            );
            let known_revision = match revision {
                RevisionPlan::Existing { revision } => Some(revision.clone()),
                RevisionPlan::Resolve { .. } | RevisionPlan::Reuse { .. } => None,
            };
            world.deployments.insert(
                deployment.clone(),
                Deployment {
                    id: deployment.clone(),
                    service: service.clone(),
                    revision: known_revision,
                    plan: revision.clone(),
                    phase: DeploymentPhase::Resolving,
                    reason: reason.clone(),
                    author: author.clone(),
                    created_at: at,
                    started_at: None,
                    healthy_at: None,
                    finished_at: None,
                    secrets: Vec::new(),
                },
            );
            touch_service_project(world, service, at);
        }
        Event::RevisionCreated {
            revision,
            service,
            number,
            spec,
        } => {
            assert!(
                !world.revisions.contains_key(revision),
                "RevisionCreated for an existing revision"
            );
            let target = service_mut(world, service);
            for path in &spec.volumes {
                if !target.volumes.contains(path) {
                    target.volumes.push(path.clone());
                }
            }
            world.revisions.insert(
                revision.clone(),
                Revision {
                    id: revision.clone(),
                    service: service.clone(),
                    number: *number,
                    created_at: at,
                    spec: spec.clone(),
                },
            );
        }
        Event::DeploymentStarted {
            deployment,
            revision,
            secrets,
        } => {
            assert!(
                world.revisions.contains_key(revision),
                "DeploymentStarted with an unknown revision"
            );
            let target = deployment_mut(world, deployment);
            assert_eq!(
                target.phase,
                DeploymentPhase::Resolving,
                "DeploymentStarted for a deployment that is not resolving"
            );
            target.revision = Some(revision.clone());
            target.secrets = secrets.clone();
            target.phase = DeploymentPhase::Starting;
            target.started_at = Some(at);
        }
        Event::DeploymentHealthy { deployment } => {
            let target = deployment_mut(world, deployment);
            assert_eq!(
                target.phase,
                DeploymentPhase::Starting,
                "DeploymentHealthy for a deployment that is not starting"
            );
            target.phase = DeploymentPhase::Promoting;
            target.healthy_at = Some(at);
        }
        Event::DeploymentSucceeded { deployment } => {
            let target = deployment_mut(world, deployment);
            assert_eq!(
                target.phase,
                DeploymentPhase::Promoting,
                "DeploymentSucceeded for a deployment that is not promoting"
            );
            let service = target.service.clone();
            for other in world.deployments.values_mut() {
                if other.service == service && other.phase == DeploymentPhase::Active {
                    other.phase = DeploymentPhase::Superseded;
                }
            }
            let target = deployment_mut(world, deployment);
            target.phase = DeploymentPhase::Active;
            target.finished_at = Some(at);
        }
        Event::DeploymentFailed { deployment, reason } => {
            let target = deployment_mut(world, deployment);
            assert!(
                target.phase.is_in_progress(),
                "DeploymentFailed for a deployment that is not in progress"
            );
            target.phase = DeploymentPhase::Failed(reason.clone());
            target.finished_at = Some(at);
        }
        Event::IngressConfigured { zone, tunnel_id } => {
            world.ingress = Some(Ingress {
                zone: zone.clone(),
                tunnel_id: tunnel_id.clone(),
            });
        }
        Event::SecretRevealed { .. }
        | Event::LoginSucceeded { .. }
        | Event::LoginFailed
        | Event::SessionClosed { .. } => {}
    }
}

fn service_mut<'a>(world: &'a mut World, service: &ServiceId) -> &'a mut Service {
    world
        .services
        .get_mut(service)
        .expect("an event about an unknown service")
}

fn deployment_mut<'a>(world: &'a mut World, deployment: &DeploymentId) -> &'a mut Deployment {
    world
        .deployments
        .get_mut(deployment)
        .expect("an event about an unknown deployment")
}

fn touch_project(world: &mut World, project: &ProjectId, at: Timestamp) {
    if let Some(project) = world.projects.get_mut(project) {
        project.updated_at = project.updated_at.max(at);
    }
}

fn touch_service_project(world: &mut World, service: &ServiceId, at: Timestamp) {
    if let Some(project) = world.project_of_service(service).cloned() {
        touch_project(world, &project, at);
    }
}

fn touch_owner_project(world: &mut World, owner: &VariableOwner, at: Timestamp) {
    let project = match owner {
        VariableOwner::Project(project) => Some(project.clone()),
        VariableOwner::Environment(environment) => {
            world.project_of_environment(environment).cloned()
        }
    };
    if let Some(project) = project {
        touch_project(world, &project, at);
    }
}
