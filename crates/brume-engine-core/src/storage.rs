//! What the engine core needs from durable storage. The shell implements it on SQLite; tests,
//! in memory.

use brume_domain::{
    AdminId, BlobId, ChangeId, EnvironmentId, ServiceId, Timestamp, VarName, VariableId,
};
use brume_proto::api::{SettingField, VariableKindDto};
use secrecy::SecretString;

/// Durable storage: the event log, and the tables outside it.
pub trait Storage {
    /// Every event, by increasing `seq`.
    fn load_events(&self) -> Result<Vec<StoredEvent>, StorageError>;

    /// Writes the whole transaction, or nothing.
    fn commit(&mut self, tx: Transaction) -> Result<(), StorageError>;

    /// A blob's bytes.
    fn blob(&self, id: &BlobId) -> Result<Option<Vec<u8>>, StorageError>;

    /// A secret's value at a version.
    fn secret(
        &self,
        variable: &VariableId,
        version: u32,
    ) -> Result<Option<SecretString>, StorageError>;

    /// Every staged change, oldest first.
    fn staged_changes(&self) -> Result<Vec<StoredChange>, StorageError>;

    /// The session or login link whose token hashes to `token_hash`.
    fn session(&self, token_hash: &[u8; 32]) -> Result<Option<StoredSession>, StorageError>;

    /// The Cloudflare tunnel's token, once the public entry is set up.
    fn ingress_token(&self) -> Result<Option<SecretString>, StorageError>;

    /// At most `limit` events with a `seq` below `before` (or the newest ones), newest first.
    fn events_page(
        &self,
        before: Option<u64>,
        limit: u32,
    ) -> Result<Vec<StoredEvent>, StorageError>;
}

/// A storage failure. The message never holds a value.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("storage failed: {0}")]
pub struct StorageError(pub String);

/// An event as the log stores it: its compact JSON envelope and its chained hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredEvent {
    pub seq: u64,
    pub bytes: Vec<u8>,
    /// `SHA-256(previous hash ‖ bytes)`.
    pub hash: [u8; 32],
}

/// Everything one decision writes, atomically.
#[derive(Debug, Default)]
pub struct Transaction {
    pub events: Vec<StoredEvent>,
    pub blobs: Vec<(BlobId, Vec<u8>)>,
    /// The implementation encrypts them.
    pub secrets: Vec<(VariableId, u32, SecretString)>,
    pub staged_upserts: Vec<StoredChange>,
    pub staged_deletes: Vec<ChangeId>,
    pub sessions_upserts: Vec<StoredSession>,
    pub sessions_deletes: Vec<[u8; 32]>,
    /// Replaces the tunnel's token. The implementation encrypts it.
    pub ingress_token: Option<SecretString>,
}

/// A change waiting for the deploy of its environment.
#[derive(Clone, Debug)]
pub struct StoredChange {
    pub id: ChangeId,
    pub environment: EnvironmentId,
    pub service: ServiceId,
    pub body: StagedBody,
}

/// What a staged change does.
#[derive(Clone, Debug)]
pub enum StagedBody {
    /// A setting; `value` is the panel's `SettingValue`, already validated.
    Setting {
        field: SettingField,
        value: serde_json::Value,
    },
    /// A variable or secret of the service. A secret without a value references the shared
    /// secret of that name.
    Variable {
        name: VarName,
        kind: VariableKindDto,
        value: Option<StagedValue>,
    },
    VariableRemoval {
        name: VarName,
    },
}

/// The value of a staged variable. The implementation encrypts a secret.
#[derive(Clone, Debug)]
pub enum StagedValue {
    Plain(String),
    Secret(SecretString),
}

/// A session, or a one-time login link, by the SHA-256 of its token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredSession {
    pub token_hash: [u8; 32],
    pub admin: AdminId,
    pub kind: SessionKind,
    pub expires_at: Timestamp,
}

/// What a token opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SessionKind {
    /// A logged-in panel.
    Session,
    /// A one-time link that opens a session.
    LoginLink,
}
