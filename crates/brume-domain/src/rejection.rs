//! Why `decide` refuses a command.

/// The class of a refusal. The API maps it to an HTTP status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RejectionKind {
    /// The target does not exist.
    NotFound,
    /// The command breaks a validation rule.
    Invalid,
    /// The command conflicts with the current state.
    Conflict,
}

/// A command that `decide` refuses. The message is an English sentence for the panel, and it
/// never contains a value.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct Rejection {
    /// The class of refusal.
    pub kind: RejectionKind,
    /// An English sentence, with a final period.
    pub message: String,
}

impl Rejection {
    /// The target does not exist.
    pub fn not_found(message: String) -> Self {
        Self {
            kind: RejectionKind::NotFound,
            message,
        }
    }

    /// The command breaks a validation rule.
    pub fn invalid(message: String) -> Self {
        Self {
            kind: RejectionKind::Invalid,
            message,
        }
    }

    /// The command conflicts with the current state.
    pub fn conflict(message: String) -> Self {
        Self {
            kind: RejectionKind::Conflict,
            message,
        }
    }
}
