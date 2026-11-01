//! The engine's `Storage` on SQLite: the event log and the tables outside it. WAL with
//! `synchronous=FULL`, so a commit is durable. Secrets, and the tunnel token, are sealed with the
//! master key, bound to their row. Every query is parameterized.

use std::fs::{DirBuilder, OpenOptions, Permissions};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

use brume_domain::{
    AdminId, BlobId, ChangeId, EnvironmentId, ServiceId, Timestamp, VarName, VariableId,
};
use brume_engine_core::{
    SessionKind, StagedBody, StagedValue, Storage, StorageError, StoredChange, StoredEvent,
    StoredSession, Transaction,
};
use brume_proto::api::{SettingField, VariableKindDto};
use brume_runtime::crypto::{MasterKey, open, seal};
use rusqlite::{Connection, OptionalExtension, params};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// How long a writer waits for the shipper's connection, and the other way round.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS events (
        seq INTEGER PRIMARY KEY,
        bytes BLOB NOT NULL,
        hash BLOB NOT NULL
    );
    CREATE TABLE IF NOT EXISTS blobs (
        id TEXT PRIMARY KEY,
        bytes BLOB NOT NULL
    );
    CREATE TABLE IF NOT EXISTS secret_versions (
        variable TEXT NOT NULL,
        version INTEGER NOT NULL,
        sealed BLOB NOT NULL,
        PRIMARY KEY (variable, version)
    );
    CREATE TABLE IF NOT EXISTS staged_changes (
        id TEXT PRIMARY KEY,
        json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS sessions (
        token_hash BLOB PRIMARY KEY,
        admin TEXT NOT NULL,
        kind TEXT NOT NULL,
        expires_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS ingress_secrets (
        name TEXT PRIMARY KEY,
        sealed BLOB NOT NULL
    );
    CREATE TABLE IF NOT EXISTS shipper (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        next_seq INTEGER NOT NULL
    );
";

/// A storage failure, without any value: SQL is parameterized, so SQLite's messages hold none.
fn failed(error: impl std::fmt::Display) -> StorageError {
    StorageError(error.to_string())
}

/// Opens a connection with the engine's pragmas and schema.
pub(crate) fn connect(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    connection.execute_batch(SCHEMA)?;
    Ok(connection)
}

/// The engine's storage.
pub struct SqliteStorage {
    connection: Connection,
    key: MasterKey,
}

/// A staged change as `staged_changes.json` stores it: a staged secret is sealed.
#[derive(Serialize, Deserialize)]
struct ChangeRow {
    environment: String,
    service: String,
    body: BodyRow,
}

#[derive(Serialize, Deserialize)]
enum BodyRow {
    Setting {
        field: SettingField,
        value: serde_json::Value,
    },
    Variable {
        name: String,
        kind: VariableKindDto,
        value: Option<ValueRow>,
    },
    VariableRemoval {
        name: String,
    },
}

#[derive(Serialize, Deserialize)]
enum ValueRow {
    Plain(String),
    /// Hexadecimal of the sealed value.
    Sealed(String),
}

fn secret_aad(variable: &VariableId, version: u32) -> Vec<u8> {
    format!("secret_versions:{variable}:{version}").into_bytes()
}

/// The row of `ingress_secrets` that holds the Cloudflare tunnel's token.
const TUNNEL_TOKEN: &str = "tunnel_token";

fn ingress_aad(name: &str) -> Vec<u8> {
    format!("ingress_secrets:{name}").into_bytes()
}

fn staged_aad(change: &ChangeId) -> Vec<u8> {
    format!("staged_changes:{change}").into_bytes()
}

fn kind_name(kind: SessionKind) -> &'static str {
    match kind {
        SessionKind::Session => "session",
        SessionKind::LoginLink => "login_link",
    }
}

/// The files of the database at `path`: the database itself, its WAL and its shared memory.
fn database_files(path: &Path) -> [PathBuf; 3] {
    let with_suffix = |suffix: &str| {
        let mut name = path.as_os_str().to_owned();
        name.push(suffix);
        PathBuf::from(name)
    };
    [path.to_path_buf(), with_suffix("-wal"), with_suffix("-shm")]
}

/// Restricts each file of the database at `path` that exists to its owner: they hold the log,
/// the staged changes and the sealed secrets.
fn restrict(path: &Path) -> std::io::Result<()> {
    for file in database_files(path) {
        match std::fs::set_permissions(&file, Permissions::from_mode(0o600)) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    Ok(())
}

impl SqliteStorage {
    /// Opens or creates the database at `path`, and its directory, readable by their owner
    /// only, whatever the umask: a database that an older version left readable is restricted.
    pub fn open(path: &Path, key: MasterKey) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)
                .map_err(failed)?;
        }
        // Created before SQLite opens it, which gives its WAL files the same mode.
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(path)
            .map_err(failed)?;
        restrict(path).map_err(failed)?;
        let connection = connect(path).map_err(failed)?;
        restrict(path).map_err(failed)?;
        Ok(Self { connection, key })
    }

    fn change_json(&self, change: &StoredChange) -> Result<String, StorageError> {
        let body = match &change.body {
            StagedBody::Setting { field, value } => BodyRow::Setting {
                field: *field,
                value: value.clone(),
            },
            StagedBody::Variable { name, kind, value } => BodyRow::Variable {
                name: name.to_string(),
                kind: *kind,
                value: value.as_ref().map(|value| match value {
                    StagedValue::Plain(value) => ValueRow::Plain(value.clone()),
                    StagedValue::Secret(secret) => ValueRow::Sealed(hex::encode(seal(
                        &self.key,
                        &staged_aad(&change.id),
                        secret.expose_secret().as_bytes(),
                    ))),
                }),
            },
            StagedBody::VariableRemoval { name } => BodyRow::VariableRemoval {
                name: name.to_string(),
            },
        };
        serde_json::to_string(&ChangeRow {
            environment: change.environment.to_string(),
            service: change.service.to_string(),
            body,
        })
        .map_err(failed)
    }

    fn change_from_row(&self, id: &str, json: &str) -> Result<StoredChange, StorageError> {
        let corrupt = || StorageError(format!("staged change {id} is corrupt"));
        let id = ChangeId::parse(id).map_err(|_| corrupt())?;
        let row: ChangeRow = serde_json::from_str(json).map_err(|_| corrupt())?;
        let name = |raw: &str| VarName::parse(raw).map_err(|_| corrupt());
        let body = match row.body {
            BodyRow::Setting { field, value } => StagedBody::Setting { field, value },
            BodyRow::Variable {
                name: raw,
                kind,
                value,
            } => StagedBody::Variable {
                name: name(&raw)?,
                kind,
                value: match value {
                    None => None,
                    Some(ValueRow::Plain(value)) => Some(StagedValue::Plain(value)),
                    Some(ValueRow::Sealed(sealed)) => {
                        let sealed = hex::decode(sealed).map_err(|_| corrupt())?;
                        let plain =
                            open(&self.key, &staged_aad(&id), &sealed).map_err(|_| corrupt())?;
                        let plain = String::from_utf8(plain).map_err(|_| corrupt())?;
                        Some(StagedValue::Secret(SecretString::from(plain)))
                    }
                },
            },
            BodyRow::VariableRemoval { name: raw } => {
                StagedBody::VariableRemoval { name: name(&raw)? }
            }
        };
        Ok(StoredChange {
            id,
            environment: EnvironmentId::parse(&row.environment).map_err(|_| corrupt())?,
            service: ServiceId::parse(&row.service).map_err(|_| corrupt())?,
            body,
        })
    }
}

fn event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(i64, Vec<u8>, Vec<u8>)> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
}

fn to_event((seq, bytes, hash): (i64, Vec<u8>, Vec<u8>)) -> Result<StoredEvent, StorageError> {
    let corrupt = || StorageError(format!("event {seq} is corrupt"));
    let hash = <[u8; 32]>::try_from(hash.as_slice()).map_err(|_| corrupt())?;
    let seq = u64::try_from(seq).map_err(|_| corrupt())?;
    Ok(StoredEvent { seq, bytes, hash })
}

/// SQLite integers are signed: a `u64` past `i64::MAX` cannot be stored.
pub(crate) fn sql_int(value: u64) -> Result<i64, StorageError> {
    i64::try_from(value).map_err(|_| StorageError(format!("{value} is too large for SQLite")))
}

impl Storage for SqliteStorage {
    fn load_events(&self) -> Result<Vec<StoredEvent>, StorageError> {
        let mut statement = self
            .connection
            .prepare("SELECT seq, bytes, hash FROM events ORDER BY seq")
            .map_err(failed)?;
        let rows = statement
            .query_map([], event_row)
            .map_err(failed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(failed)?;
        rows.into_iter().map(to_event).collect()
    }

    fn commit(&mut self, tx: Transaction) -> Result<(), StorageError> {
        // Sealing first: the SQL transaction then only writes.
        let secrets = tx
            .secrets
            .iter()
            .map(|(variable, version, secret)| {
                let sealed = seal(
                    &self.key,
                    &secret_aad(variable, *version),
                    secret.expose_secret().as_bytes(),
                );
                (variable.to_string(), *version, sealed)
            })
            .collect::<Vec<_>>();
        let changes = tx
            .staged_upserts
            .iter()
            .map(|change| Ok((change.id.to_string(), self.change_json(change)?)))
            .collect::<Result<Vec<_>, StorageError>>()?;
        let ingress_token = tx.ingress_token.as_ref().map(|token| {
            seal(
                &self.key,
                &ingress_aad(TUNNEL_TOKEN),
                token.expose_secret().as_bytes(),
            )
        });

        // Dropping the transaction on an error rolls it back.
        let sql = self.connection.transaction().map_err(failed)?;
        for event in &tx.events {
            sql.execute(
                "INSERT INTO events (seq, bytes, hash) VALUES (?1, ?2, ?3)",
                params![sql_int(event.seq)?, event.bytes, event.hash.as_slice()],
            )
            .map_err(failed)?;
        }
        for (id, bytes) in &tx.blobs {
            sql.execute(
                "INSERT INTO blobs (id, bytes) VALUES (?1, ?2)",
                params![id.as_str(), bytes],
            )
            .map_err(failed)?;
        }
        for (variable, version, sealed) in &secrets {
            sql.execute(
                "INSERT INTO secret_versions (variable, version, sealed) VALUES (?1, ?2, ?3)",
                params![variable, version, sealed],
            )
            .map_err(failed)?;
        }
        for (id, json) in &changes {
            sql.execute(
                "INSERT INTO staged_changes (id, json) VALUES (?1, ?2)
                 ON CONFLICT (id) DO UPDATE SET json = excluded.json",
                params![id, json],
            )
            .map_err(failed)?;
        }
        for id in &tx.staged_deletes {
            sql.execute(
                "DELETE FROM staged_changes WHERE id = ?1",
                params![id.as_str()],
            )
            .map_err(failed)?;
        }
        for session in &tx.sessions_upserts {
            sql.execute(
                "INSERT INTO sessions (token_hash, admin, kind, expires_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (token_hash) DO UPDATE SET
                     admin = excluded.admin,
                     kind = excluded.kind,
                     expires_at = excluded.expires_at",
                params![
                    session.token_hash.as_slice(),
                    session.admin.as_str(),
                    kind_name(session.kind),
                    sql_int(session.expires_at.as_millis())?,
                ],
            )
            .map_err(failed)?;
        }
        for token_hash in &tx.sessions_deletes {
            sql.execute(
                "DELETE FROM sessions WHERE token_hash = ?1",
                params![token_hash.as_slice()],
            )
            .map_err(failed)?;
        }
        if let Some(sealed) = &ingress_token {
            sql.execute(
                "INSERT INTO ingress_secrets (name, sealed) VALUES (?1, ?2)
                 ON CONFLICT (name) DO UPDATE SET sealed = excluded.sealed",
                params![TUNNEL_TOKEN, sealed],
            )
            .map_err(failed)?;
        }
        sql.commit().map_err(failed)
    }

    fn blob(&self, id: &BlobId) -> Result<Option<Vec<u8>>, StorageError> {
        self.connection
            .query_row(
                "SELECT bytes FROM blobs WHERE id = ?1",
                params![id.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(failed)
    }

    fn secret(
        &self,
        variable: &VariableId,
        version: u32,
    ) -> Result<Option<SecretString>, StorageError> {
        let sealed: Option<Vec<u8>> = self
            .connection
            .query_row(
                "SELECT sealed FROM secret_versions WHERE variable = ?1 AND version = ?2",
                params![variable.as_str(), version],
                |row| row.get(0),
            )
            .optional()
            .map_err(failed)?;
        let Some(sealed) = sealed else {
            return Ok(None);
        };
        let corrupt = || {
            StorageError(format!(
                "version {version} of secret {variable} does not open"
            ))
        };
        let plain =
            open(&self.key, &secret_aad(variable, version), &sealed).map_err(|_| corrupt())?;
        let plain = String::from_utf8(plain).map_err(|_| corrupt())?;
        Ok(Some(SecretString::from(plain)))
    }

    fn staged_changes(&self) -> Result<Vec<StoredChange>, StorageError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, json FROM staged_changes ORDER BY id")
            .map_err(failed)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(failed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(failed)?;
        rows.iter()
            .map(|(id, json)| self.change_from_row(id, json))
            .collect()
    }

    fn session(&self, token_hash: &[u8; 32]) -> Result<Option<StoredSession>, StorageError> {
        let row: Option<(String, String, i64)> = self
            .connection
            .query_row(
                "SELECT admin, kind, expires_at FROM sessions WHERE token_hash = ?1",
                params![token_hash.as_slice()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(failed)?;
        let Some((admin, kind, expires_at)) = row else {
            return Ok(None);
        };
        let corrupt = || StorageError("a session row is corrupt".to_owned());
        let kind = match kind.as_str() {
            "session" => SessionKind::Session,
            "login_link" => SessionKind::LoginLink,
            _ => return Err(corrupt()),
        };
        Ok(Some(StoredSession {
            token_hash: *token_hash,
            admin: AdminId::parse(&admin).map_err(|_| corrupt())?,
            kind,
            expires_at: Timestamp::from_millis(u64::try_from(expires_at).map_err(|_| corrupt())?),
        }))
    }

    fn ingress_token(&self) -> Result<Option<SecretString>, StorageError> {
        let sealed: Option<Vec<u8>> = self
            .connection
            .query_row(
                "SELECT sealed FROM ingress_secrets WHERE name = ?1",
                params![TUNNEL_TOKEN],
                |row| row.get(0),
            )
            .optional()
            .map_err(failed)?;
        let Some(sealed) = sealed else {
            return Ok(None);
        };
        let corrupt = || StorageError("the tunnel token does not open".to_owned());
        let plain = open(&self.key, &ingress_aad(TUNNEL_TOKEN), &sealed).map_err(|_| corrupt())?;
        let plain = String::from_utf8(plain).map_err(|_| corrupt())?;
        Ok(Some(SecretString::from(plain)))
    }

    fn events_page(
        &self,
        before: Option<u64>,
        limit: u32,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT seq, bytes, hash FROM events WHERE seq < ?1 ORDER BY seq DESC LIMIT ?2",
            )
            .map_err(failed)?;
        let rows = statement
            .query_map(
                params![
                    before.map_or(i64::MAX, |before| i64::try_from(before).unwrap_or(i64::MAX)),
                    limit
                ],
                event_row,
            )
            .map_err(failed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(failed)?;
        rows.into_iter().map(to_event).collect()
    }
}
