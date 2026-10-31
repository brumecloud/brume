//! Shipping the log to the bucket: contiguous segments, resumed without gaps or duplicates,
//! never over another server's objects.

use std::path::Path;

use brume_engine::{ShipOutcome, Shipper, SqliteStorage, segment_bytes, segment_key};
use brume_engine_core::{Storage, StoredEvent, Transaction};
use brume_runtime::crypto::MasterKey;
use brume_runtime::object_store::{DirStore, ObjectStore, PutOutcome};

fn db(home: &Path) -> std::path::PathBuf {
    home.join("engine/state.sqlite")
}

fn storage(home: &Path) -> SqliteStorage {
    SqliteStorage::open(&db(home), MasterKey::from_bytes([3; 32])).expect("opens")
}

fn store(home: &Path) -> DirStore {
    DirStore::new(home.join("bucket"))
}

fn commit(storage: &mut SqliteStorage, seqs: std::ops::RangeInclusive<u64>) {
    let events = seqs
        .map(|seq| StoredEvent {
            seq,
            bytes: format!(r#"{{"seq":{seq},"type":"Test","note":"é"}}"#).into_bytes(),
            hash: [0; 32],
        })
        .collect();
    storage
        .commit(Transaction {
            events,
            ..Transaction::default()
        })
        .expect("commits");
}

/// Every event line of every segment, in key order, and the segments' headers.
fn shipped(store: &DirStore) -> (Vec<Vec<u8>>, Vec<serde_json::Value>) {
    let mut lines = Vec::new();
    let mut headers = Vec::new();
    for key in store.list("log/").expect("lists") {
        let bytes = store.get(&key).expect("reads").expect("exists");
        assert_eq!(bytes.last(), Some(&b'\n'));
        let mut parts = bytes[..bytes.len() - 1].split(|byte| *byte == b'\n');
        let header = parts.next().expect("a header");
        headers.push(serde_json::from_slice(header).expect("a JSON header"));
        lines.extend(parts.map(<[u8]>::to_vec));
    }
    (lines, headers)
}

fn stored(storage: &SqliteStorage) -> Vec<Vec<u8>> {
    storage
        .load_events()
        .expect("loads")
        .into_iter()
        .map(|event| event.bytes)
        .collect()
}

#[test]
fn commits_ship_as_contiguous_segments() {
    let home = tempfile::tempdir().expect("a temporary directory");
    let mut storage = storage(home.path());
    let mut shipper = Shipper::open(&db(home.path()), store(home.path())).expect("opens");
    assert_eq!(shipper.ship_once().expect("ships"), ShipOutcome::Idle);

    commit(&mut storage, 1..=2);
    assert_eq!(
        shipper.ship_once().expect("ships"),
        ShipOutcome::Shipped {
            first_seq: 1,
            count: 2
        }
    );
    commit(&mut storage, 3..=3);
    commit(&mut storage, 4..=6);
    assert_eq!(
        shipper.ship_once().expect("ships"),
        ShipOutcome::Shipped {
            first_seq: 3,
            count: 4
        }
    );
    assert_eq!(shipper.ship_once().expect("ships"), ShipOutcome::Idle);

    let store = store(home.path());
    assert_eq!(
        store.list("log/").expect("lists"),
        [
            "log/0000000001/00000000000000000001.seg",
            "log/0000000001/00000000000000000003.seg",
        ]
    );
    let (lines, headers) = shipped(&store);
    assert_eq!(lines, stored(&storage));
    assert_eq!(
        headers,
        [
            serde_json::json!({"segment": 1, "generation": 1, "first_seq": 1, "count": 2}),
            serde_json::json!({"segment": 1, "generation": 1, "first_seq": 3, "count": 4}),
        ]
    );
    let first = store
        .get("log/0000000001/00000000000000000001.seg")
        .expect("reads")
        .expect("exists");
    assert!(first.starts_with(br#"{"segment":1,"generation":1,"first_seq":1,"count":2}"#));
}

#[test]
fn restart_resumes_without_gap_or_duplicate() {
    let home = tempfile::tempdir().expect("a temporary directory");
    let mut storage = storage(home.path());
    commit(&mut storage, 1..=3);
    let mut shipper = Shipper::open(&db(home.path()), store(home.path())).expect("opens");
    shipper.ship_once().expect("ships");
    drop(shipper);

    // A crash after the put of seq 4 and 5, before the checkpoint: the object is ours.
    commit(&mut storage, 4..=5);
    let pending = stored(&storage)[3..].to_vec();
    assert_eq!(
        store(home.path())
            .put_if_absent(&segment_key(1, 4), &segment_bytes(1, 4, &pending))
            .expect("writes"),
        PutOutcome::Created
    );
    commit(&mut storage, 6..=7);

    let mut shipper = Shipper::open(&db(home.path()), store(home.path())).expect("reopens");
    assert_eq!(
        shipper.ship_once().expect("ships"),
        ShipOutcome::Shipped {
            first_seq: 4,
            count: 2
        }
    );
    assert_eq!(
        shipper.ship_once().expect("ships"),
        ShipOutcome::Shipped {
            first_seq: 6,
            count: 2
        }
    );
    assert_eq!(shipper.ship_once().expect("ships"), ShipOutcome::Idle);
    let (lines, _) = shipped(&store(home.path()));
    assert_eq!(lines, stored(&storage));
}

#[test]
fn another_servers_object_is_a_conflict() {
    let home = tempfile::tempdir().expect("a temporary directory");
    let mut storage = storage(home.path());
    commit(&mut storage, 1..=2);
    store(home.path())
        .put_if_absent(
            &segment_key(1, 1),
            b"{\"segment\":1,\"generation\":1,\"first_seq\":1,\"count\":2}\nother\nlog\n",
        )
        .expect("writes");

    let mut shipper = Shipper::open(&db(home.path()), store(home.path())).expect("opens");
    assert_eq!(
        shipper.ship_once().expect("checks"),
        ShipOutcome::Conflict {
            key: segment_key(1, 1)
        }
    );
    // Nothing was overwritten, and the checkpoint did not move.
    assert_eq!(
        store(home.path()).get(&segment_key(1, 1)).expect("reads"),
        Some(
            b"{\"segment\":1,\"generation\":1,\"first_seq\":1,\"count\":2}\nother\nlog\n".to_vec()
        )
    );
    assert!(matches!(
        shipper.ship_once().expect("checks"),
        ShipOutcome::Conflict { .. }
    ));
}
