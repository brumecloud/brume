//! The messages on `admin.sock`, which only local commands such as `brume dev` use.

use serde::{Deserialize, Serialize};

/// A request from a local command to the engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminRequest {
    /// A one-time login link for the admin.
    CreateLoginLink,
}

/// The engine's answer to an `AdminRequest`. Its `Debug` masks the token.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminReply {
    /// The link's token, to put in the panel's `/login#token=` URL.
    LoginLink {
        /// Base64url.
        token: String,
        /// RFC 3339.
        expires_at: String,
    },
}
