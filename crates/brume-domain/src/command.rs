//! What `decide` is asked to do. The engine has already drawn the identifiers and written the
//! blobs and secret versions that a command names.

use crate::ids::{
    AdminId, BlobId, DeploymentId, EnvironmentId, MachineId, ProjectId, RevisionId, ServiceId,
    VariableId,
};
use crate::model::{FailureReason, PlainKind, SharedValue, VariableOwner};
use crate::names::{
    ContainerPath, HttpPath, ImageDigestRef, ImageRef, ProjectName, ServiceName, Slug,
    StartCommand, Subdomain, VarName, Zone,
};
use crate::values::{CpuMillis, MemoryMb, Port};

/// A command for `decide`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// The first start: the machine and its admin.
    Bootstrap {
        machine: MachineId,
        admin: AdminId,
        /// The admin's identity, in a blob.
        identity: BlobId,
    },
    CreateProject {
        project: ProjectId,
        name: ProjectName,
        /// The id of the `production` environment, created with the project.
        production: EnvironmentId,
        /// A first service in `production`.
        first_service: Option<NewService>,
    },
    CreateEnvironment {
        environment: EnvironmentId,
        project: ProjectId,
        name: Slug,
    },
    /// The first deployment is requested in the same decision.
    CreateService(NewService),
    /// Fails the service's deployment in progress, if any, then deletes the service.
    DeleteService { service: ServiceId },
    /// The staged changes of an environment, one deployment per service.
    DeployChanges {
        environment: EnvironmentId,
        services: Vec<ServiceChanges>,
        author: AdminId,
    },
    /// Deploys the active revision again, or, when no deployment ever succeeded, the service as
    /// it is now.
    Redeploy {
        service: ServiceId,
        deployment: DeploymentId,
        author: AdminId,
    },
    /// Deploys the revision of `target` again, and puts the service's spec back to it.
    Rollback {
        target: DeploymentId,
        deployment: DeploymentId,
        author: AdminId,
    },
    /// Creates or replaces a shared variable. A plain one redeploys each service in
    /// `deployments`: exactly the impacted services.
    SetSharedVariable {
        variable: VariableId,
        owner: VariableOwner,
        name: VarName,
        value: SharedValue,
        deployments: Vec<(ServiceId, DeploymentId)>,
        author: AdminId,
    },
    /// Removes a shared variable. A plain one redeploys each service in `deployments`: exactly
    /// the impacted services.
    RemoveSharedVariable {
        variable: VariableId,
        deployments: Vec<(ServiceId, DeploymentId)>,
        author: AdminId,
    },
    /// The image of a resolving deployment resolved, or its revision is reused.
    ImageResolved {
        deployment: DeploymentId,
        revision: RevisionId,
        digest: ImageDigestRef,
        volumes: Vec<ContainerPath>,
    },
    ImageResolutionFailed {
        deployment: DeploymentId,
        /// Docker's error, already cut to 1 KiB.
        message: String,
    },
    /// The starting container passed its healthcheck.
    DeploymentHealthy { deployment: DeploymentId },
    /// The promoting container holds the service's alias.
    DeploymentPromoted { deployment: DeploymentId },
    DeploymentFailed {
        deployment: DeploymentId,
        reason: FailureReason,
    },
    /// Sets up, or sets up again, the public entry. The engine stores the tunnel's token
    /// beside the event, sealed.
    ConfigureIngress { zone: Zone, tunnel_id: String },
    /// An audit fact, which does not change the `World`.
    RecordAudit(AuditEvent),
}

/// A service to create, with its first deployment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewService {
    pub service: ServiceId,
    pub environment: EnvironmentId,
    pub name: ServiceName,
    pub slug: Slug,
    pub image: ImageRef,
    pub deployment: DeploymentId,
    pub author: AdminId,
}

/// The staged changes of one service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceChanges {
    pub service: ServiceId,
    pub deployment: DeploymentId,
    pub changes: Vec<SpecChange>,
}

/// One change to a service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecChange {
    Image(ImageRef),
    StartCommand(Option<StartCommand>),
    Port(Option<Port>),
    HealthcheckPath(Option<HttpPath>),
    Cpu(CpuMillis),
    Memory(MemoryMb),
    Egress(bool),
    /// The public subdomain. It needs the public entry and the service's port, and no other
    /// service may hold it.
    Domain(Option<Subdomain>),
    /// Sets a plain or build variable, whose value the engine wrote in `blob`. `variable` is
    /// the id of the service's variable of that name, or a new one.
    PlainVariable {
        name: VarName,
        variable: VariableId,
        kind: PlainKind,
        blob: BlobId,
    },
    /// Sets a secret of the service, whose value the engine wrote at `version`.
    OwnSecret {
        name: VarName,
        variable: VariableId,
        version: u32,
    },
    /// References the shared secret of that name.
    SharedSecretReference {
        name: VarName,
        variable: VariableId,
    },
    RemoveVariable {
        name: VarName,
    },
}

/// A sensitive action that does not change the state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditEvent {
    SecretRevealed {
        variable: VariableId,
        admin: AdminId,
    },
    LoginSucceeded {
        admin: AdminId,
    },
    LoginFailed,
    SessionClosed {
        admin: AdminId,
    },
}
