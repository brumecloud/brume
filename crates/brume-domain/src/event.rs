//! The facts of the log. An event never holds a value: variables point to blobs and secrets to
//! versions, both in the engine's store.

use serde::{Deserialize, Serialize};

use crate::ids::{
    AdminId, BlobId, DeploymentId, EnvironmentId, MachineId, ProjectId, RevisionId, ServiceId,
    VariableId,
};
use crate::model::{
    DeployReason, FailureReason, ResolvedSecret, RevisionSpec, ServiceSpec, SharedValue,
    VariableOwner,
};
use crate::names::{
    ContainerPath, ImageDigestRef, ImageRef, ProjectName, ServiceName, Slug, VarName,
};
use crate::time::Timestamp;

/// The version of every event type: no type is published before the first release.
pub const SCHEMA_VERSION: u16 = 1;

/// The `prev_hash` of the first event: 64 hexadecimal zeros.
pub const GENESIS_PREV_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// An event and its metadata, as the log stores it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Its position in the log, from 1, without gaps.
    pub seq: u64,
    pub generation: u32,
    /// The entity it is about: `"<kind>:<id>"`.
    pub stream: String,
    pub schema_version: u16,
    pub at: Timestamp,
    /// `"admin:<AdminId>"`, `"agent:<MachineId>"` or `"engine"`.
    pub actor: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    /// The hash of the previous event, in lowercase hexadecimal.
    pub prev_hash: String,
    #[serde(flatten)]
    pub event: Event,
}

/// How a deployment obtains its revision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RevisionPlan {
    /// An existing revision, deployed again.
    Existing { revision: RevisionId },
    /// A new revision, once the agent resolves the image to a digest.
    Resolve { image: ImageRef },
    /// A new revision with the digest and volumes of the active one: only the settings or the
    /// variables changed.
    Reuse {
        digest: ImageDigestRef,
        volumes: Vec<ContainerPath>,
    },
}

/// A fact of the log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    MachineRegistered {
        machine: MachineId,
    },
    AdminCreated {
        admin: AdminId,
        identity: BlobId,
    },
    ProjectCreated {
        project: ProjectId,
        name: ProjectName,
    },
    EnvironmentCreated {
        environment: EnvironmentId,
        project: ProjectId,
        name: Slug,
    },
    ServiceCreated {
        service: ServiceId,
        environment: EnvironmentId,
        name: ServiceName,
        slug: Slug,
        spec: ServiceSpec,
    },
    ServiceDeleted {
        service: ServiceId,
    },
    ServiceSpecChanged {
        service: ServiceId,
        spec: ServiceSpec,
    },
    ServiceEgressChanged {
        service: ServiceId,
        egress: bool,
    },
    SharedVariableSet {
        variable: VariableId,
        owner: VariableOwner,
        name: VarName,
        value: SharedValue,
    },
    SharedVariableRemoved {
        variable: VariableId,
    },
    /// A new version of a service's own secret.
    SecretVersionAdded {
        variable: VariableId,
        version: u32,
    },
    DeploymentRequested {
        deployment: DeploymentId,
        service: ServiceId,
        revision: RevisionPlan,
        reason: DeployReason,
        author: AdminId,
    },
    RevisionCreated {
        revision: RevisionId,
        service: ServiceId,
        number: u32,
        spec: RevisionSpec,
    },
    DeploymentStarted {
        deployment: DeploymentId,
        revision: RevisionId,
        secrets: Vec<ResolvedSecret>,
    },
    DeploymentHealthy {
        deployment: DeploymentId,
    },
    DeploymentSucceeded {
        deployment: DeploymentId,
    },
    DeploymentFailed {
        deployment: DeploymentId,
        reason: FailureReason,
    },
    // Audit events: they do not change the `World`.
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

impl Event {
    /// The event's type, as its `type` field writes it.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::MachineRegistered { .. } => "MachineRegistered",
            Self::AdminCreated { .. } => "AdminCreated",
            Self::ProjectCreated { .. } => "ProjectCreated",
            Self::EnvironmentCreated { .. } => "EnvironmentCreated",
            Self::ServiceCreated { .. } => "ServiceCreated",
            Self::ServiceDeleted { .. } => "ServiceDeleted",
            Self::ServiceSpecChanged { .. } => "ServiceSpecChanged",
            Self::ServiceEgressChanged { .. } => "ServiceEgressChanged",
            Self::SharedVariableSet { .. } => "SharedVariableSet",
            Self::SharedVariableRemoved { .. } => "SharedVariableRemoved",
            Self::SecretVersionAdded { .. } => "SecretVersionAdded",
            Self::DeploymentRequested { .. } => "DeploymentRequested",
            Self::RevisionCreated { .. } => "RevisionCreated",
            Self::DeploymentStarted { .. } => "DeploymentStarted",
            Self::DeploymentHealthy { .. } => "DeploymentHealthy",
            Self::DeploymentSucceeded { .. } => "DeploymentSucceeded",
            Self::DeploymentFailed { .. } => "DeploymentFailed",
            Self::SecretRevealed { .. } => "SecretRevealed",
            Self::LoginSucceeded { .. } => "LoginSucceeded",
            Self::LoginFailed => "LoginFailed",
            Self::SessionClosed { .. } => "SessionClosed",
        }
    }

    /// The entity it is about: `"<kind>:<id>"`.
    pub fn stream(&self) -> String {
        match self {
            Self::MachineRegistered { machine } => format!("machine:{machine}"),
            Self::AdminCreated { admin, .. }
            | Self::LoginSucceeded { admin }
            | Self::SessionClosed { admin } => format!("admin:{admin}"),
            Self::ProjectCreated { project, .. } => format!("project:{project}"),
            Self::EnvironmentCreated { environment, .. } => format!("environment:{environment}"),
            Self::ServiceCreated { service, .. }
            | Self::ServiceDeleted { service }
            | Self::ServiceSpecChanged { service, .. }
            | Self::ServiceEgressChanged { service, .. }
            | Self::RevisionCreated { service, .. } => format!("service:{service}"),
            Self::SharedVariableSet { variable, .. }
            | Self::SharedVariableRemoved { variable }
            | Self::SecretVersionAdded { variable, .. }
            | Self::SecretRevealed { variable, .. } => format!("variable:{variable}"),
            Self::DeploymentRequested { deployment, .. }
            | Self::DeploymentStarted { deployment, .. }
            | Self::DeploymentHealthy { deployment }
            | Self::DeploymentSucceeded { deployment }
            | Self::DeploymentFailed { deployment, .. } => format!("deployment:{deployment}"),
            Self::LoginFailed => "session".to_owned(),
        }
    }

    /// Whether it is an audit event, which does not change the `World`.
    pub fn is_audit(&self) -> bool {
        matches!(
            self,
            Self::SecretRevealed { .. }
                | Self::LoginSucceeded { .. }
                | Self::LoginFailed
                | Self::SessionClosed { .. }
        )
    }
}
