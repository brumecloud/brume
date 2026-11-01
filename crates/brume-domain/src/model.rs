//! The `World`: the current state, rebuilt by replaying the event log. Only `evolve` changes it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::event::RevisionPlan;
use crate::ids::{
    AdminId, BlobId, DeploymentId, EnvironmentId, MachineId, ProjectId, RevisionId, ServiceId,
    VariableId,
};
use crate::names::{
    ContainerPath, HttpPath, ImageDigestRef, ImageRef, ProjectName, ServiceName, Slug,
    StartCommand, Subdomain, VarName, Zone,
};
use crate::time::Timestamp;
use crate::values::{CpuMillis, MemoryMb, Port};

/// The CPU limit of a new service: one CPU.
pub const DEFAULT_CPU_MILLIS: u32 = 1000;

/// The memory limit of a new service, in MiB.
pub const DEFAULT_MEMORY_MB: u32 = 512;

/// A project: environments that share variables and secrets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub name: ProjectName,
    pub description: String,
    pub created_at: Timestamp,
    /// The instant of the last event about the project or one of its entities.
    pub updated_at: Timestamp,
}

/// An environment of a project, with its own Docker network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Environment {
    pub id: EnvironmentId,
    pub project: ProjectId,
    /// Unique in the project.
    pub name: Slug,
}

/// A service of an environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Service {
    pub id: ServiceId,
    pub environment: EnvironmentId,
    pub name: ServiceName,
    /// Its host name on the environment network: immutable, unique among the live services of
    /// the environment.
    pub slug: Slug,
    /// Outbound internet access.
    pub egress: bool,
    /// The public subdomain, under the zone of the public entry. A link with the outside, like
    /// `egress`: a rollback keeps it, since its DNS record stays.
    pub domain: Option<Subdomain>,
    /// Every path that one of its revisions declared as a volume, in the order they appeared:
    /// it names the volumes. The volume of the path at index `n` is the service's volume `n`,
    /// so the list only grows, even with the path of a revision that never ran. What a
    /// container mounts, and whether a deployment stops the active container first, comes
    /// from the revisions that run, never from this list.
    pub volumes: Vec<ContainerPath>,
    /// What the next deployment starts from.
    pub spec: ServiceSpec,
    /// Deleted services stay in the `World` for the history.
    pub deleted: bool,
}

/// The editable spec of a service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub image: ImageRef,
    pub start_command: Option<StartCommand>,
    pub port: Option<Port>,
    pub healthcheck_path: Option<HttpPath>,
    pub cpu: CpuMillis,
    pub memory: MemoryMb,
    /// The service-level variables and secrets.
    pub variables: BTreeMap<VarName, ServiceVariable>,
}

impl ServiceSpec {
    /// The spec of a new service: the image and the default limits.
    pub fn new(image: ImageRef) -> Self {
        Self {
            image,
            start_command: None,
            port: None,
            healthcheck_path: None,
            cpu: CpuMillis::new(DEFAULT_CPU_MILLIS).expect("the default CPU limit is valid"),
            memory: MemoryMb::new(DEFAULT_MEMORY_MB).expect("the default memory limit is valid"),
            variables: BTreeMap::new(),
        }
    }
}

/// A variable or secret at the service level.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceVariable {
    pub id: VariableId,
    pub value: ServiceVariableValue,
}

/// The value of a service-level variable. Values live in the engine's store, never here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServiceVariableValue {
    /// A plain or build variable, whose value is a blob.
    Plain { kind: PlainKind, blob: BlobId },
    /// A secret of the service, at this version.
    OwnSecret { version: u32 },
    /// A reference, by name, to a shared secret of the environment or the project.
    SharedSecret,
}

impl ServiceVariableValue {
    /// Whether the name is a secret.
    pub fn is_secret(&self) -> bool {
        !matches!(self, Self::Plain { .. })
    }
}

/// A variable that is not a secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PlainKind {
    /// Injected in the container.
    Plain,
    /// Part of the build. Injected too in this milestone, which only deploys images.
    Build,
}

/// A variable or secret at the project or environment level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedVariable {
    pub id: VariableId,
    pub owner: VariableOwner,
    pub name: VarName,
    pub value: SharedValue,
}

/// What holds a shared variable.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VariableOwner {
    Project(ProjectId),
    Environment(EnvironmentId),
}

/// The value of a shared variable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SharedValue {
    /// A plain or build variable, whose value is a blob.
    Plain { kind: PlainKind, blob: BlobId },
    /// A secret, at this version.
    Secret { version: u32 },
}

impl SharedValue {
    /// Whether the name is a secret.
    pub fn is_secret(&self) -> bool {
        matches!(self, Self::Secret { .. })
    }
}

/// An immutable spec that deployments put in service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    pub id: RevisionId,
    pub service: ServiceId,
    /// 1, 2, 3… for each service.
    pub number: u32,
    pub created_at: Timestamp,
    pub spec: RevisionSpec,
}

/// What a revision runs: the image pinned by digest, the settings and the resolved variables.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionSpec {
    /// The reference that resolved to `digest`.
    pub image: ImageRef,
    pub digest: ImageDigestRef,
    /// The paths that the image declares as volumes.
    pub volumes: Vec<ContainerPath>,
    pub start_command: Option<StartCommand>,
    pub port: Option<Port>,
    pub healthcheck_path: Option<HttpPath>,
    pub cpu: CpuMillis,
    pub memory: MemoryMb,
    /// Sorted by name.
    pub variables: Vec<RevisionVariable>,
}

/// A variable of a revision, with where its value came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionVariable {
    pub name: VarName,
    pub origin: Level,
    pub value: RevisionValue,
}

/// A level of variables. The most precise one wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Level {
    Project,
    Environment,
    Service,
}

/// The value of a revision variable. A plain value is resolved and kept; a secret is resolved
/// at each deployment, so only its name is kept.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RevisionValue {
    /// `variable` is the service or shared variable that gave the value: a rollback restores
    /// the service-level ones under their id.
    Plain {
        kind: PlainKind,
        blob: BlobId,
        variable: VariableId,
    },
    /// The service-level entry that brought the secret in.
    Secret { source: SecretSource },
}

/// The service-level entry of a revision secret. Its value is resolved at each deployment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SecretSource {
    /// The service's own secret.
    Own { variable: VariableId },
    /// The service's reference to a shared secret of the same name.
    Reference { variable: VariableId },
}

/// A deployment of a service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deployment {
    pub id: DeploymentId,
    pub service: ServiceId,
    /// Known once the deployment starts.
    pub revision: Option<RevisionId>,
    /// How the revision is obtained.
    pub plan: RevisionPlan,
    pub phase: DeploymentPhase,
    pub reason: DeployReason,
    pub author: AdminId,
    pub created_at: Timestamp,
    /// When its container was asked to start.
    pub started_at: Option<Timestamp>,
    /// When its container passed the healthcheck.
    pub healthy_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    /// The secret versions resolved when it started.
    pub secrets: Vec<ResolvedSecret>,
}

/// Where a deployment stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeploymentPhase {
    /// The image is being resolved to a digest.
    Resolving,
    /// The new container starts; it has no alias yet.
    Starting,
    /// The container is healthy and takes the service's alias.
    Promoting,
    /// The container serves the service.
    Active,
    /// A later deployment replaced it.
    Superseded,
    Failed(FailureReason),
}

impl DeploymentPhase {
    /// Resolving, starting or promoting.
    pub fn is_in_progress(&self) -> bool {
        matches!(self, Self::Resolving | Self::Starting | Self::Promoting)
    }
}

/// Why a deployment failed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FailureReason {
    /// The image could not be pulled. The message is Docker's, cut to 1 KiB.
    ImageResolutionFailed {
        message: String,
    },
    ContainerExited {
        code: i64,
    },
    HealthcheckFailed {
        reason: String,
    },
    /// The deployment did not finish in time.
    Timeout,
    /// The service was deleted while it deployed.
    Replaced,
    /// Some secrets did not resolve when it started.
    MissingSecrets {
        names: Vec<VarName>,
    },
}

/// Why a deployment was requested.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeployReason {
    /// The first deployment of a new service.
    Created,
    /// Staged changes were deployed.
    Changes,
    /// The active revision, deployed again.
    Redeploy,
    /// The revision of an earlier deployment, deployed again.
    Rollback { target: DeploymentId },
    /// A shared plain variable changed or was removed.
    SharedVariable { variable: VariableId },
}

/// A secret, resolved to a version when a deployment starts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSecret {
    pub name: VarName,
    pub variable: VariableId,
    pub version: u32,
}

/// The public entry, set up by `brume install --domain`: a Cloudflare tunnel to Traefik. The
/// tunnel's token lives in the engine's store, sealed, never here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ingress {
    /// The zone whose subdomains the services get.
    pub zone: Zone,
    /// The Cloudflare tunnel.
    pub tunnel_id: String,
}

impl Ingress {
    /// The public host name of a subdomain: `<subdomain>.<zone>`.
    pub fn host(&self, subdomain: &Subdomain) -> String {
        format!("{subdomain}.{}", self.zone)
    }
}

/// A variable found by its id: shared, or held by a service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableRef<'a> {
    Shared(&'a SharedVariable),
    Service {
        service: &'a Service,
        name: &'a VarName,
        variable: &'a ServiceVariable,
    },
}

/// The current state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct World {
    pub(crate) machine: Option<MachineId>,
    pub(crate) admins: BTreeMap<AdminId, BlobId>,
    pub(crate) projects: BTreeMap<ProjectId, Project>,
    pub(crate) environments: BTreeMap<EnvironmentId, Environment>,
    pub(crate) services: BTreeMap<ServiceId, Service>,
    pub(crate) revisions: BTreeMap<RevisionId, Revision>,
    pub(crate) deployments: BTreeMap<DeploymentId, Deployment>,
    pub(crate) shared_variables: BTreeMap<VariableId, SharedVariable>,
    pub(crate) ingress: Option<Ingress>,
}

impl World {
    /// An empty world, before the first event.
    pub fn new() -> Self {
        Self::default()
    }

    /// The machine of the agent, once registered.
    pub fn machine(&self) -> Option<&MachineId> {
        self.machine.as_ref()
    }

    /// The public entry, once configured.
    pub fn ingress(&self) -> Option<&Ingress> {
        self.ingress.as_ref()
    }

    /// The admins, with the blob of their identity.
    pub fn admins(&self) -> impl Iterator<Item = (&AdminId, &BlobId)> {
        self.admins.iter()
    }

    pub fn project(&self, id: &ProjectId) -> Option<&Project> {
        self.projects.get(id)
    }

    pub fn projects(&self) -> impl Iterator<Item = &Project> {
        self.projects.values()
    }

    pub fn environment(&self, id: &EnvironmentId) -> Option<&Environment> {
        self.environments.get(id)
    }

    pub fn environments(&self) -> impl Iterator<Item = &Environment> {
        self.environments.values()
    }

    /// The environments of a project.
    pub fn environments_of(&self, project: &ProjectId) -> impl Iterator<Item = &Environment> {
        self.environments
            .values()
            .filter(move |environment| &environment.project == project)
    }

    /// A service, live or deleted.
    pub fn service(&self, id: &ServiceId) -> Option<&Service> {
        self.services.get(id)
    }

    /// A service that is not deleted.
    pub fn live_service(&self, id: &ServiceId) -> Option<&Service> {
        self.services.get(id).filter(|service| !service.deleted)
    }

    /// Every service, deleted ones included.
    pub fn services(&self) -> impl Iterator<Item = &Service> {
        self.services.values()
    }

    /// The live services of an environment.
    pub fn services_in(&self, environment: &EnvironmentId) -> impl Iterator<Item = &Service> {
        self.services
            .values()
            .filter(move |service| !service.deleted && &service.environment == environment)
    }

    /// The live services of every environment of a project.
    pub fn services_of_project(&self, project: &ProjectId) -> impl Iterator<Item = &Service> {
        self.services.values().filter(move |service| {
            !service.deleted
                && self
                    .environments
                    .get(&service.environment)
                    .is_some_and(|environment| &environment.project == project)
        })
    }

    /// The project of an environment.
    pub fn project_of_environment(&self, environment: &EnvironmentId) -> Option<&ProjectId> {
        self.environments
            .get(environment)
            .map(|environment| &environment.project)
    }

    /// The project of a service.
    pub fn project_of_service(&self, service: &ServiceId) -> Option<&ProjectId> {
        self.services
            .get(service)
            .and_then(|service| self.project_of_environment(&service.environment))
    }

    pub fn deployment(&self, id: &DeploymentId) -> Option<&Deployment> {
        self.deployments.get(id)
    }

    /// Every deployment, oldest id first.
    pub fn deployments(&self) -> impl Iterator<Item = &Deployment> {
        self.deployments.values()
    }

    /// The deployments of a service.
    pub fn deployments_of(&self, service: &ServiceId) -> impl Iterator<Item = &Deployment> {
        self.deployments
            .values()
            .filter(move |deployment| &deployment.service == service)
    }

    pub fn revision(&self, id: &RevisionId) -> Option<&Revision> {
        self.revisions.get(id)
    }

    /// The revisions of a service.
    pub fn revisions_of(&self, service: &ServiceId) -> impl Iterator<Item = &Revision> {
        self.revisions
            .values()
            .filter(move |revision| &revision.service == service)
    }

    /// A shared variable or a service variable.
    pub fn variable(&self, id: &VariableId) -> Option<VariableRef<'_>> {
        if let Some(shared) = self.shared_variables.get(id) {
            return Some(VariableRef::Shared(shared));
        }
        self.services
            .values()
            .filter(|service| !service.deleted)
            .find_map(|service| {
                service
                    .spec
                    .variables
                    .iter()
                    .find(|(_, variable)| &variable.id == id)
                    .map(|(name, variable)| VariableRef::Service {
                        service,
                        name,
                        variable,
                    })
            })
    }

    pub fn shared_variable(&self, id: &VariableId) -> Option<&SharedVariable> {
        self.shared_variables.get(id)
    }

    pub fn shared_variables(&self) -> impl Iterator<Item = &SharedVariable> {
        self.shared_variables.values()
    }

    /// The shared variable of an owner with that name.
    pub fn shared_variable_named(
        &self,
        owner: &VariableOwner,
        name: &VarName,
    ) -> Option<&SharedVariable> {
        self.shared_variables
            .values()
            .find(|variable| &variable.owner == owner && &variable.name == name)
    }

    /// The active deployment of a service.
    pub fn active_deployment(&self, service: &ServiceId) -> Option<&Deployment> {
        self.deployments_of(service)
            .find(|deployment| deployment.phase == DeploymentPhase::Active)
    }

    /// The deployment of a service that is resolving, starting or promoting.
    pub fn in_progress_deployment(&self, service: &ServiceId) -> Option<&Deployment> {
        self.deployments_of(service)
            .find(|deployment| deployment.phase.is_in_progress())
    }

    /// Every deployment that is resolving, starting or promoting.
    pub fn deployments_in_progress(&self) -> impl Iterator<Item = &Deployment> {
        self.deployments
            .values()
            .filter(|deployment| deployment.phase.is_in_progress())
    }

    /// The revision of the active deployment of a service.
    pub fn active_revision(&self, service: &ServiceId) -> Option<&Revision> {
        self.active_deployment(service)
            .and_then(|deployment| deployment.revision.as_ref())
            .and_then(|revision| self.revisions.get(revision))
    }

    /// A live service other than `except` that holds `subdomain`. The route follows the
    /// service's setting, so a subdomain is free as soon as its service gives it up.
    pub fn domain_holder(&self, subdomain: &Subdomain, except: &ServiceId) -> Option<&Service> {
        self.services
            .values()
            .filter(|service| !service.deleted && &service.id != except)
            .find(|service| service.domain.as_ref() == Some(subdomain))
    }

    /// The live service of an environment with that slug.
    pub fn service_by_slug(&self, environment: &EnvironmentId, slug: &Slug) -> Option<&Service> {
        self.services_in(environment)
            .find(|service| &service.slug == slug)
    }
}
