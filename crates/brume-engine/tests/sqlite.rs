//! `SqliteStorage`: transactions are atomic, and secrets are sealed at rest.

use std::path::Path;

use brume_domain::{
    AdminId, BlobId, ChangeId, EnvironmentId, ServiceId, Timestamp, VarName, VariableId,
};
use brume_engine::SqliteStorage;
use brume_engine_core::{
    SessionKind, StagedBody, StagedValue, Storage, StoredChange, StoredEvent, StoredSession,
    Transaction,
};
use brume_proto::api::{SettingField, VariableKindDto};
use brume_runtime::crypto::MasterKey;
use secrecy::{ExposeSecret, SecretString};

const SECRET: &str = "s3cr3t-🚀";

fn open(home: &Path) -> SqliteStorage {
    SqliteStorage::open(
        &home.join("engine/state.sqlite"),
        MasterKey::from_bytes([3; 32]),
    )
    .expect("opens")
}

fn id(n: u8) -> String {
    format!("0190a3f2-7c1e-7b3a-9f00-1234567890{n:02x}")
}

fn event(seq: u64) -> StoredEvent {
    StoredEvent {
        seq,
        bytes: format!(r#"{{"seq":{seq},"type":"Test"}}"#).into_bytes(),
        hash: [u8::try_from(seq).expect("small"); 32],
    }
}

fn blob(n: u8) -> BlobId {
    BlobId::parse(&format!("{n:032x}")).expect("a blob id")
}

fn change(n: u8, body: StagedBody) -> StoredChange {
    StoredChange {
        id: ChangeId::parse(&id(n)).expect("an id"),
        environment: EnvironmentId::parse(&id(100)).expect("an id"),
        service: ServiceId::parse(&id(101)).expect("an id"),
        body,
    }
}

fn session(n: u8) -> StoredSession {
    StoredSession {
        token_hash: [n; 32],
        admin: AdminId::parse(&id(102)).expect("an id"),
        kind: SessionKind::Session,
        expires_at: Timestamp::from_millis(1_793_450_096_789),
    }
}

fn variable() -> VariableId {
    VariableId::parse(&id(103)).expect("an id")
}

fn name(raw: &str) -> VarName {
    VarName::parse(raw).expect("a name")
}

fn file_bytes(home: &Path) -> Vec<u8> {
    let mut bytes = Vec::new();
    for suffix in ["", "-wal"] {
        let path = home.join(format!("engine/state.sqlite{suffix}"));
        if let Ok(content) = std::fs::read(path) {
            bytes.extend(content);
        }
    }
    bytes
}

#[test]
fn transaction_round_trips() {
    let home = tempfile::tempdir().expect("a temporary directory");
    let mut storage = open(home.path());
    storage
        .commit(Transaction {
            events: vec![event(1), event(2)],
            blobs: vec![(blob(1), b"hello".to_vec())],
            secrets: vec![(variable(), 1, SecretString::from(SECRET))],
            staged_upserts: vec![
                change(
                    1,
                    StagedBody::Setting {
                        field: SettingField::Port,
                        value: serde_json::json!(8080),
                    },
                ),
                change(
                    2,
                    StagedBody::Variable {
                        name: name("TOKEN"),
                        kind: VariableKindDto::Secret,
                        value: Some(StagedValue::Secret(SecretString::from(SECRET))),
                    },
                ),
                change(
                    3,
                    StagedBody::Variable {
                        name: name("GREETING"),
                        kind: VariableKindDto::Build,
                        value: Some(StagedValue::Plain("hi".to_owned())),
                    },
                ),
                change(
                    4,
                    StagedBody::Variable {
                        name: name("DB_PASSWORD"),
                        kind: VariableKindDto::Secret,
                        value: None,
                    },
                ),
                change(5, StagedBody::VariableRemoval { name: name("OLD") }),
            ],
            sessions_upserts: vec![session(1), session(2)],
            ..Transaction::default()
        })
        .expect("commits");
    drop(storage);

    let mut storage = open(home.path());
    assert_eq!(
        storage.load_events().expect("loads"),
        vec![event(1), event(2)]
    );
    assert_eq!(
        storage.events_page(None, 1).expect("a page"),
        vec![event(2)]
    );
    assert_eq!(
        storage.events_page(Some(2), 10).expect("a page"),
        vec![event(1)]
    );
    assert_eq!(
        storage.blob(&blob(1)).expect("reads"),
        Some(b"hello".to_vec())
    );
    assert_eq!(storage.blob(&blob(2)).expect("reads"), None);
    assert_eq!(
        storage
            .secret(&variable(), 1)
            .expect("reads")
            .map(|secret| secret.expose_secret().to_owned()),
        Some(SECRET.to_owned())
    );
    assert!(storage.secret(&variable(), 2).expect("reads").is_none());
    assert_eq!(storage.session(&[1; 32]).expect("reads"), Some(session(1)));

    let changes = storage.staged_changes().expect("reads");
    assert_eq!(changes.len(), 5);
    assert_eq!(
        changes[0].id,
        change(1, StagedBody::VariableRemoval { name: name("X") }).id
    );
    assert!(matches!(
        &changes[0].body,
        StagedBody::Setting { field: SettingField::Port, value } if *value == serde_json::json!(8080)
    ));
    assert!(matches!(
        &changes[1].body,
        StagedBody::Variable { kind: VariableKindDto::Secret, value: Some(StagedValue::Secret(secret)), .. }
            if secret.expose_secret() == SECRET
    ));
    assert!(matches!(
        &changes[2].body,
        StagedBody::Variable { kind: VariableKindDto::Build, value: Some(StagedValue::Plain(value)), .. }
            if value == "hi"
    ));
    assert!(matches!(
        &changes[3].body,
        StagedBody::Variable { value: None, .. }
    ));
    assert!(
        matches!(&changes[4].body, StagedBody::VariableRemoval { name } if name.as_str() == "OLD")
    );

    storage
        .commit(Transaction {
            events: vec![event(3)],
            staged_deletes: vec![changes[0].id.clone()],
            sessions_deletes: vec![[1; 32]],
            ..Transaction::default()
        })
        .expect("commits");
    assert_eq!(storage.staged_changes().expect("reads").len(), 4);
    assert_eq!(storage.session(&[1; 32]).expect("reads"), None);
    assert_eq!(storage.session(&[2; 32]).expect("reads"), Some(session(2)));
    assert_eq!(storage.load_events().expect("loads").len(), 3);
}

#[test]
fn secrets_are_sealed_on_disk() {
    let home = tempfile::tempdir().expect("a temporary directory");
    let mut storage = open(home.path());
    storage
        .commit(Transaction {
            secrets: vec![(variable(), 1, SecretString::from(SECRET))],
            staged_upserts: vec![change(
                1,
                StagedBody::Variable {
                    name: name("TOKEN"),
                    kind: VariableKindDto::Secret,
                    value: Some(StagedValue::Secret(SecretString::from(SECRET))),
                },
            )],
            ..Transaction::default()
        })
        .expect("commits");
    let bytes = file_bytes(home.path());
    assert!(!bytes.is_empty());
    let needle = "s3cr3t".as_bytes();
    assert!(!bytes.windows(needle.len()).any(|window| window == needle));
    assert!(storage.secret(&variable(), 1).expect("reads").is_some());

    // The same store with another key cannot read them.
    drop(storage);
    let other = SqliteStorage::open(
        &home.path().join("engine/state.sqlite"),
        MasterKey::from_bytes([4; 32]),
    )
    .expect("opens");
    assert!(other.secret(&variable(), 1).is_err());

    let connection =
        rusqlite::Connection::open(home.path().join("engine/state.sqlite")).expect("opens");
    let mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("a mode");
    assert_eq!(mode, "wal");
}

#[test]
fn failed_transaction_leaves_nothing() {
    let home = tempfile::tempdir().expect("a temporary directory");
    let mut storage = open(home.path());
    storage
        .commit(Transaction {
            events: vec![event(1)],
            secrets: vec![(variable(), 1, SecretString::from(SECRET))],
            ..Transaction::default()
        })
        .expect("commits");
    let failed = storage.commit(Transaction {
        events: vec![event(2), event(3)],
        blobs: vec![(blob(1), b"hello".to_vec())],
        // Version 2 goes in, then version 1 again fails, after the events and the blob.
        secrets: vec![
            (variable(), 2, SecretString::from("new")),
            (variable(), 1, SecretString::from("again")),
        ],
        staged_upserts: vec![change(1, StagedBody::VariableRemoval { name: name("X") })],
        sessions_upserts: vec![session(1)],
        ..Transaction::default()
    });
    assert!(failed.is_err());
    assert_eq!(storage.load_events().expect("loads"), vec![event(1)]);
    assert_eq!(storage.blob(&blob(1)).expect("reads"), None);
    assert!(storage.secret(&variable(), 2).expect("reads").is_none());
    assert_eq!(
        storage
            .secret(&variable(), 1)
            .expect("reads")
            .map(|secret| secret.expose_secret().to_owned()),
        Some(SECRET.to_owned())
    );
    assert!(storage.staged_changes().expect("reads").is_empty());
    assert_eq!(storage.session(&[1; 32]).expect("reads"), None);
}

#[test]
fn database_files_are_readable_by_their_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let home = tempfile::tempdir().expect("a temporary directory");
    let path = home.path().join("engine/state.sqlite");
    let mut storage = open(home.path());
    storage
        .commit(Transaction {
            events: vec![event(1)],
            ..Transaction::default()
        })
        .expect("commits");
    let mode = |path: &Path| {
        std::fs::metadata(path)
            .expect("exists")
            .permissions()
            .mode()
            & 0o777
    };
    let wal = path.with_file_name("state.sqlite-wal");
    let shm = path.with_file_name("state.sqlite-shm");
    for file in [&path, &wal, &shm] {
        assert_eq!(mode(file), 0o600, "{}", file.display());
    }
    assert_eq!(mode(&home.path().join("engine")), 0o700);

    // A database left readable by an older version is restricted when it opens.
    drop(storage);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");
    let _storage = open(home.path());
    assert_eq!(mode(&path), 0o600);
}
