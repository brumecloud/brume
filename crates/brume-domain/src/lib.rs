//! Brume's domain: the validated newtypes every other crate builds on. A value of one of these
//! types has passed its allowlist, so code that holds a `Slug` or an `ImageRef` never checks it
//! again. No I/O: the clock and randomness come in through `Ctx`.

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

mod ctx;
mod ids;
mod names;
mod time;
mod values;

pub use crate::time::Timestamp;
pub use ctx::Ctx;
pub use ids::{
    AdminId, BlobId, ChangeId, DeploymentId, EnvironmentId, MachineId, ProjectId, RevisionId,
    ServiceId, VariableId,
};
pub use names::{
    ContainerPath, HttpPath, ImageDigestRef, ImageRef, ProjectName, ServiceName, Slug,
    StartCommand, VarName,
};
pub use values::{CpuMillis, MAX_VALUE_BYTES, MemoryMb, PlainValue, Port, SecretValue};
