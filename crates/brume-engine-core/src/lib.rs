//! The engine's state machine. It replays the event log into a `World`, turns API
//! calls and agent reports into commands, and returns what the shell must do. No I/O: the
//! shell supplies time, randomness and storage.

#![forbid(unsafe_code)]

mod activity;
mod api;
mod deployments;
mod desired;
mod engine;
mod queries;
mod sessions;
mod staged;
mod storage;

pub use activity::MAX_EVENTS_PAGE;
pub use api::LOG_BACKLOG_LINES;
pub use deployments::DEPLOYMENT_TIMEOUT_MILLIS;
pub use desired::{
    HTTP_HEALTHCHECK_TIMEOUT_SECS, RUNNING_GRACE_SECS, container_name, desired_state,
    egress_network, environment_network, volume_name,
};
pub use engine::{ClientId, Engine, Input, OpenError, Output};
pub use queries::{derive_slug, image_base_name};
pub use sessions::{LOGIN_LINK_MILLIS, SESSION_MILLIS, token_hash};
pub use storage::{
    SessionKind, StagedBody, StagedValue, Storage, StorageError, StoredChange, StoredEvent,
    StoredSession, Transaction,
};
