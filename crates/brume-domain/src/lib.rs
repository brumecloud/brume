//! Brume's domain: the validated newtypes every other crate builds on, the `World`, and the pure
//! `decide` and `evolve` of the event-sourced engine. A value of one of the newtypes has passed
//! its allowlist, so code that holds a `Slug` or an `ImageRef` never checks it again. No I/O: the
//! clock and randomness come in through `Ctx`.

#![forbid(unsafe_code)]

/// A value refused by a newtype's allowlist. The message never repeats the value, which may be
/// a secret.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Invalid {field}: {message}.")]
pub struct ValidationError {
    /// What was being parsed, in English: `"slug"`, `"image"`, `"port"`…
    pub field: &'static str,
    /// The rule the value breaks, in English, without a leading capital or a final period.
    pub message: String,
}

impl ValidationError {
    fn new(field: &'static str, message: &str) -> Self {
        Self {
            field,
            message: message.to_owned(),
        }
    }
}

/// Implements `as_str`, `Display`, `Serialize` and a validating `Deserialize` for a newtype
/// whose field `0` is a `String` and which has `parse(&str) -> Result<Self, ValidationError>`.
macro_rules! string_newtype {
    ($name:ident) => {
        impl $name {
            /// The validated string.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = String::deserialize(deserializer)?;
                Self::parse(&raw).map_err(serde::de::Error::custom)
            }
        }
    };
}

mod command;
mod ctx;
mod decide;
mod event;
mod evolve;
mod ids;
mod model;
mod names;
mod rejection;
mod time;
mod values;

pub use crate::time::Timestamp;
pub use command::{AuditEvent, Command, NewService, ServiceChanges, SpecChange};
pub use ctx::Ctx;
pub use decide::{
    decide, impacted_services, resolve_revision_variables, resolve_secrets, secret_references,
};
pub use event::{Event, EventEnvelope, GENESIS_PREV_HASH, RevisionPlan, SCHEMA_VERSION};
pub use evolve::evolve;
pub use ids::{
    AdminId, BlobId, ChangeId, DeploymentId, EnvironmentId, MachineId, ProjectId, RevisionId,
    ServiceId, VariableId,
};
pub use model::{
    DEFAULT_CPU_MILLIS, DEFAULT_MEMORY_MB, DeployReason, Deployment, DeploymentPhase, Environment,
    FailureReason, Level, PlainKind, Project, ResolvedSecret, Revision, RevisionSpec,
    RevisionValue, RevisionVariable, Service, ServiceSpec, ServiceVariable, ServiceVariableValue,
    SharedValue, SharedVariable, VariableOwner, VariableRef, World,
};
pub use names::{
    ContainerPath, HttpPath, ImageDigestRef, ImageRef, ProjectName, ServiceName, Slug,
    StartCommand, VarName,
};
pub use rejection::{Rejection, RejectionKind};
pub use values::{CpuMillis, MAX_VALUE_BYTES, MemoryMb, PlainValue, Port, SecretValue};
