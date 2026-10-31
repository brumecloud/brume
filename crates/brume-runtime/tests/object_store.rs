//! The local bucket: objects are written once, never overwritten.

use std::io::ErrorKind;

use brume_runtime::object_store::{DirStore, ObjectStore, PutOutcome};

#[test]
fn put_if_absent_never_overwrites() {
    let root = tempfile::tempdir().expect("a temporary directory");
    let store = DirStore::new(root.path().to_path_buf());
    let key = "log/0000000001/00000000000000000001.seg";
    assert_eq!(
        store.put_if_absent(key, b"first").expect("writes"),
        PutOutcome::Created
    );
    assert_eq!(
        store.put_if_absent(key, b"second").expect("checks"),
        PutOutcome::AlreadyExists
    );
    assert_eq!(store.get(key).expect("reads"), Some(b"first".to_vec()));
    assert_eq!(store.get("log/missing.seg").expect("reads"), None);
    // No temporary file is left behind.
    let names = std::fs::read_dir(root.path().join("log/0000000001"))
        .expect("lists")
        .map(|entry| entry.expect("an entry").file_name())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 1);
}

#[test]
fn unsafe_keys_are_refused() {
    let root = tempfile::tempdir().expect("a temporary directory");
    let store = DirStore::new(root.path().join("bucket"));
    for key in [
        "../x",
        "log/../../x",
        "/etc/passwd",
        "",
        "log//x",
        "log/",
        "Log/x",
        "log/x y",
        "log/é",
        "log\\x",
    ] {
        let error = store.put_if_absent(key, b"x").expect_err(key);
        assert_eq!(error.kind(), ErrorKind::InvalidInput, "{key:?}");
        assert_eq!(
            store.get(key).expect_err(key).kind(),
            ErrorKind::InvalidInput
        );
    }
    assert!(!root.path().join("x").exists());
}

#[test]
fn list_is_sorted_and_filtered_by_prefix() {
    let root = tempfile::tempdir().expect("a temporary directory");
    let store = DirStore::new(root.path().to_path_buf());
    for key in [
        "log/0000000001/00000000000000000010.seg",
        "log/0000000001/00000000000000000002.seg",
        "log/0000000002/00000000000000000001.seg",
        "blobs/abc",
    ] {
        store.put_if_absent(key, key.as_bytes()).expect("writes");
    }
    assert_eq!(
        store.list("log/").expect("lists"),
        [
            "log/0000000001/00000000000000000002.seg",
            "log/0000000001/00000000000000000010.seg",
            "log/0000000002/00000000000000000001.seg",
        ]
    );
    assert_eq!(store.list("blobs/").expect("lists"), ["blobs/abc"]);
    assert!(store.list("nothing/").expect("lists").is_empty());
}
