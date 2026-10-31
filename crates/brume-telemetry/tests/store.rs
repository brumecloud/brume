//! The DuckDB store, the level heuristic and the two sockets of the telemetry service.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use brume_proto::api::LogLevel;
use brume_proto::telemetry::{LogBatch, LogQuery, LogStreamFrame, RawLogLine, StdStream};
use brume_telemetry::framed::{FramedReader, FramedWriter};
use brume_telemetry::level::classify;
use brume_telemetry::paths::Paths;
use brume_telemetry::store::{HistoryQuery, Store};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

fn raw(environment_id: &str, timestamp_ms: u64, message: &str) -> RawLogLine {
    RawLogLine {
        timestamp_ms,
        environment_id: environment_id.to_owned(),
        service_id: "svc".to_owned(),
        deployment_id: "dep".to_owned(),
        stream: StdStream::Stdout,
        message: message.to_owned(),
    }
}

/// A home under `/tmp`: macOS limits a unix socket path to 104 bytes.
fn home() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("brume-tel")
        .tempdir_in("/tmp")
        .expect("creates a temporary home")
}

#[test]
fn classify_finds_whole_words_without_case() {
    assert_eq!(classify(StdStream::Stdout, "GET / 200"), LogLevel::Info);
    assert_eq!(classify(StdStream::Stdout, "[warn] slow"), LogLevel::Warn);
    assert_eq!(classify(StdStream::Stdout, "WARNING: disk"), LogLevel::Warn);
    assert_eq!(classify(StdStream::Stderr, "Error: boom"), LogLevel::Error);
    assert_eq!(
        classify(StdStream::Stdout, "a FATAL thing"),
        LogLevel::Error
    );
    assert_eq!(classify(StdStream::Stdout, "panic!"), LogLevel::Error);
    assert_eq!(
        classify(StdStream::Stdout, "Exception in thread"),
        LogLevel::Error
    );
    assert_eq!(
        classify(StdStream::Stdout, "warn then error"),
        LogLevel::Error
    );
    assert_eq!(classify(StdStream::Stdout, "terrorist"), LogLevel::Info);
    assert_eq!(classify(StdStream::Stderr, "forewarned"), LogLevel::Info);
}

#[test]
fn store_reads_back_lines_filtered_and_in_order() {
    let home = home();
    let mut store = Store::open(&home.path().join("logs.duckdb")).expect("opens");
    let stored = store
        .insert(vec![
            raw("env-a", 2_000, "second"),
            raw("env-b", 1_500, "other"),
            raw("env-a", 1_000, "Error: first"),
        ])
        .expect("inserts");
    let ids: Vec<u64> = stored.iter().map(|line| line.id).collect();
    assert_eq!(ids, vec![1, 2, 3]);

    let history = |environment_id: &str, since_ms, limit| {
        store
            .history(&HistoryQuery {
                environment_id: environment_id.to_owned(),
                since_ms,
                after_id: None,
                limit,
            })
            .expect("reads")
    };
    let env_a: Vec<String> = history("env-a", None, 10)
        .iter()
        .map(|line| line.message.clone())
        .collect();
    assert_eq!(env_a, vec!["Error: first", "second"]);

    let first = &history("env-a", None, 10)[0];
    assert_eq!(first.id, 3);
    assert_eq!(first.level, LogLevel::Error);
    let dto = first.to_dto();
    assert_eq!(dto.id, "3");
    assert_eq!(dto.timestamp, "1970-01-01T00:00:01.000Z");
    assert_eq!(dto.service_id, "svc");
    assert_eq!(dto.level, LogLevel::Error);

    // The limit keeps the latest lines, still in chronological order.
    let latest: Vec<String> = history("env-a", None, 1)
        .iter()
        .map(|line| line.message.clone())
        .collect();
    assert_eq!(latest, vec!["second"]);
    let since: Vec<String> = history("env-a", Some(1_500), 10)
        .iter()
        .map(|line| line.message.clone())
        .collect();
    assert_eq!(since, vec!["second"]);
    assert_eq!(history("env-c", None, 10), vec![]);
}

#[test]
fn ids_keep_increasing_after_a_reopen() {
    let home = home();
    let path = home.path().join("logs.duckdb");
    {
        let mut store = Store::open(&path).expect("opens");
        store
            .insert(vec![raw("env", 1, "a"), raw("env", 2, "b")])
            .expect("inserts");
    }
    let mut store = Store::open(&path).expect("reopens");
    let stored = store.insert(vec![raw("env", 3, "c")]).expect("inserts");
    assert_eq!(stored[0].id, 3);
}

#[test]
fn retention_deletes_old_lines() {
    let home = home();
    let mut store = Store::open(&home.path().join("logs.duckdb")).expect("opens");
    store
        .insert(vec![raw("env", 1_000, "old"), raw("env", 9_000, "new")])
        .expect("inserts");
    assert_eq!(store.delete_before(5_000).expect("deletes"), 1);
    let kept = store
        .history(&HistoryQuery {
            environment_id: "env".to_owned(),
            since_ms: None,
            after_id: None,
            limit: 10,
        })
        .expect("reads");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].message, "new");
}

#[test]
fn external_access_is_off_and_locked() {
    let home = home();
    let store = Store::open(&home.path().join("logs.duckdb")).expect("opens");
    let target = home.path().join("x.csv");
    let copy = format!("COPY logs TO '{}'", target.display());
    assert!(store.execute_batch(&copy).is_err());
    assert!(!target.exists());
    assert!(
        store
            .execute_batch("SET enable_external_access = true")
            .is_err()
    );
    assert!(
        store
            .execute_batch("SET autoload_known_extensions = true")
            .is_err()
    );
    // A second connection, as the queries use, shares the locked settings.
    let reader = store.reader().expect("clones");
    assert!(reader.execute_batch(&copy).is_err());
    assert!(!target.exists());
}

#[test]
fn a_long_line_is_cut_on_a_char_boundary() {
    let home = home();
    let mut store = Store::open(&home.path().join("logs.duckdb")).expect("opens");
    let long = "é".repeat(brume_proto::telemetry::MAX_LOG_LINE_BYTES);
    let stored = store.insert(vec![raw("env", 1, &long)]).expect("inserts");
    assert!(stored[0].message.len() <= brume_proto::telemetry::MAX_LOG_LINE_BYTES);
    assert!(stored[0].message.chars().all(|c| c == 'é'));
}

/// Starts the service on a temporary home and waits for its sockets.
async fn start(home: &Path) -> Paths {
    let paths = Paths::new(home.to_path_buf());
    let service = tokio::spawn(brume_telemetry::run(home.to_path_buf()));
    for _ in 0..200 {
        if paths.telemetry_ingest_socket().exists() && paths.telemetry_query_socket().exists() {
            return paths;
        }
        assert!(!service.is_finished(), "the service stopped");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("the sockets never appeared");
}

async fn connect(path: PathBuf) -> (FramedReader<OwnedReadHalf>, FramedWriter<OwnedWriteHalf>) {
    let (read, write) = UnixStream::connect(path)
        .await
        .expect("connects")
        .into_split();
    (FramedReader::new(read), FramedWriter::new(write))
}

async fn next_lines(reader: &mut FramedReader<OwnedReadHalf>) -> Option<LogStreamFrame> {
    tokio::time::timeout(Duration::from_secs(5), reader.next::<LogStreamFrame>())
        .await
        .expect("a frame within 5 s")
        .expect("decodes")
}

/// Now, in milliseconds since the Unix epoch: the service deletes lines older than 7 days.
fn now_ms() -> u64 {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after 1970");
    u64::try_from(since_epoch.as_millis()).expect("fits in u64")
}

fn query(environment_id: &str, follow: bool) -> LogQuery {
    LogQuery {
        environment_id: environment_id.to_owned(),
        since_ms: None,
        limit: 100,
        follow,
    }
}

#[tokio::test]
async fn history_without_follow_ends() {
    let home = home();
    let paths = start(home.path()).await;
    let (_, mut ingest) = connect(paths.telemetry_ingest_socket()).await;
    ingest
        .send(&LogBatch {
            lines: vec![raw("env", now_ms(), "one"), raw("other", now_ms(), "skip")],
        })
        .await
        .expect("sends");

    // Ingestion has no acknowledgement: ask until the line is there.
    for _ in 0..200 {
        let (mut reader, mut writer) = connect(paths.telemetry_query_socket()).await;
        writer.send(&query("env", false)).await.expect("sends");
        match next_lines(&mut reader).await {
            Some(LogStreamFrame::Lines(lines)) => {
                let messages: Vec<&str> = lines.iter().map(|l| l.message.as_str()).collect();
                assert_eq!(messages, vec!["one"]);
                assert_eq!(next_lines(&mut reader).await, Some(LogStreamFrame::End));
                assert_eq!(next_lines(&mut reader).await, None);
                return;
            }
            Some(LogStreamFrame::End) => {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            None => panic!("closed without End"),
        }
    }
    panic!("the line never arrived");
}

#[tokio::test]
async fn follow_receives_a_line_inserted_after_it_opened() {
    let home = home();
    let paths = start(home.path()).await;
    let (_, mut ingest) = connect(paths.telemetry_ingest_socket()).await;
    ingest
        .send(&LogBatch {
            lines: vec![raw("env", now_ms(), "before")],
        })
        .await
        .expect("sends");

    let (mut reader, mut writer) = connect(paths.telemetry_query_socket()).await;
    writer.send(&query("env", true)).await.expect("sends");
    ingest
        .send(&LogBatch {
            lines: vec![
                raw("other", now_ms(), "skip"),
                raw("env", now_ms() + 1, "[warn] after"),
            ],
        })
        .await
        .expect("sends");

    let mut received = Vec::new();
    while received.len() < 2 {
        match next_lines(&mut reader).await {
            Some(LogStreamFrame::Lines(lines)) => received.extend(lines),
            other => panic!("expected lines, got {other:?}"),
        }
    }
    let messages: Vec<&str> = received.iter().map(|l| l.message.as_str()).collect();
    assert_eq!(messages, vec!["before", "[warn] after"]);
    assert_eq!(received[1].level, LogLevel::Warn);
    assert_eq!(received[1].id, "3");
}
