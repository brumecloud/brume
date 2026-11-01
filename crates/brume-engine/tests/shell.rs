//! `brume_engine::run` end to end, on real unix sockets: login, a deployment job for the
//! agent, the log stream relayed from a fake telemetry, and read-only on a bucket conflict.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use brume_engine::{GENERATION, segment_key};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_proto::agent::{AgentToEngine, EngineToAgent, Job};
use brume_proto::api::{
    ApiCall, ApiReply, ApiRequest, ApiValue, FailureKind, LogLevel, LogLineDto, NewProjectSourceDto,
};
use brume_proto::telemetry::{LogQuery, LogStreamFrame};
use brume_runtime::framed::{FramedReader, FramedWriter};
use brume_runtime::object_store::{DirStore, ObjectStore};
use brume_runtime::paths::Paths;
use brume_runtime::peer::Peers;
use secrecy::SecretString;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{UnixListener, UnixStream};
use tokio::time::timeout;

const WAIT: Duration = Duration::from_secs(10);

/// A home short enough for unix socket paths (104 bytes on macOS).
fn home() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("brume-")
        .tempdir_in("/tmp")
        .expect("a temporary directory")
}

struct Connection {
    reader: FramedReader<OwnedReadHalf>,
    writer: FramedWriter<OwnedWriteHalf>,
}

async fn connect(path: &Path) -> Connection {
    let stream = timeout(WAIT, async {
        loop {
            if let Ok(stream) = UnixStream::connect(path).await {
                return stream;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("the socket is up");
    let (read, write) = stream.into_split();
    Connection {
        reader: FramedReader::new(read),
        writer: FramedWriter::new(write),
    }
}

impl Connection {
    async fn call(&mut self, id: u64, session: Option<&str>, body: ApiCall) -> ApiReply {
        self.writer
            .write(&ApiRequest {
                id,
                session: session.map(ToOwned::to_owned),
                body,
            })
            .await
            .expect("writes");
        timeout(WAIT, self.reader.read::<ApiReply>())
            .await
            .expect("a reply in time")
            .expect("reads")
            .expect("a reply")
    }
}

/// Starts the engine, and logs in through `admin.sock` and `api.sock`.
async fn start(paths: &Paths) -> (tokio::task::JoinHandle<anyhow::Result<()>>, String) {
    let peers = Peers::same_user(&paths.run_dir()).expect("creates the run directory");
    let engine = tokio::spawn(brume_engine::run(paths.clone(), peers));
    let mut admin = connect(&paths.admin_socket()).await;
    admin
        .writer
        .write(&AdminRequest::CreateLoginLink)
        .await
        .expect("writes");
    let reply = timeout(WAIT, admin.reader.read::<AdminReply>())
        .await
        .expect("in time")
        .expect("reads")
        .expect("a reply");
    let AdminReply::LoginLink { token, .. } = reply else {
        panic!("expected a login link, got {reply:?}");
    };
    let mut api = connect(&paths.api_socket()).await;
    let reply = api.call(1, None, ApiCall::Login { token }).await;
    let Ok(ApiValue::Session { token, .. }) = reply.result else {
        panic!("login failed: {reply:?}");
    };
    (engine, token)
}

/// On a host, each socket serves its own user and closes any other's connection at once.
#[tokio::test]
async fn on_a_host_each_socket_refuses_the_other_users() {
    let home = home();
    let paths = Paths::new(home.path().to_path_buf());
    let Peers::SameUser { uid } = Peers::same_user(&paths.run_dir()).expect("creates it") else {
        panic!("the peers of a single user");
    };
    // This process is the API's user, and no other.
    let peers = Peers::System {
        engine: uid + 1,
        agent: uid + 2,
        api: uid,
        telemetry: uid + 3,
    };
    let engine = tokio::spawn(brume_engine::run(paths.clone(), peers));

    let mut api = connect(&paths.api_socket()).await;
    let reply = api.call(7, None, ApiCall::GetSession).await;
    assert_eq!(reply.id, 7);

    // Root may use `admin.sock` on a host.
    let mut refused = vec![paths.agent_socket()];
    if uid != 0 {
        refused.push(paths.admin_socket());
    }
    for socket in refused {
        let mut connection = connect(&socket).await;
        // The engine may close before the write: the write's result does not matter.
        let _ = connection
            .writer
            .write(&AdminRequest::CreateLoginLink)
            .await;
        let read = timeout(WAIT, connection.reader.read::<AdminReply>())
            .await
            .expect("closed in time");
        assert!(matches!(read, Ok(None) | Err(_)), "{}", socket.display());
    }
    for socket in [
        paths.api_socket(),
        paths.admin_socket(),
        paths.agent_socket(),
    ] {
        let mode = std::fs::metadata(&socket)
            .expect("exists")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o660, "{}", socket.display());
    }
    engine.abort();
}

#[tokio::test]
async fn engine_serves_its_sockets_and_relays_log_streams() {
    let home = home();
    let paths = Paths::new(home.path().to_path_buf());
    let (engine, session) = start(&paths).await;
    for socket in [
        paths.api_socket(),
        paths.admin_socket(),
        paths.agent_socket(),
    ] {
        let mode = std::fs::metadata(&socket)
            .expect("exists")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "{}", socket.display());
    }
    let key_mode = std::fs::metadata(paths.master_key())
        .expect("exists")
        .permissions()
        .mode();
    assert_eq!(key_mode & 0o777, 0o600);

    // A project with an image: the agent gets its job and a desired state.
    let mut api = connect(&paths.api_socket()).await;
    let reply = api
        .call(
            1,
            Some(&session),
            ApiCall::CreateProject {
                source: NewProjectSourceDto::Image {
                    image: "nginx:1.29".to_owned(),
                },
            },
        )
        .await;
    assert!(
        matches!(reply.result, Ok(ApiValue::CreatedProject(_))),
        "{reply:?}"
    );
    let reply = api.call(2, Some(&session), ApiCall::ListEnvironments).await;
    let Ok(ApiValue::Environments(environments)) = reply.result else {
        panic!("{reply:?}");
    };
    let environment = environments[0].id.clone();

    let mut agent = connect(&paths.agent_socket()).await;
    agent
        .writer
        .write(&AgentToEngine::Hello {
            version: "test".to_owned(),
            applied_position: None,
        })
        .await
        .expect("writes");
    let mut job = None;
    let mut desired = None;
    while job.is_none() || desired.is_none() {
        match timeout(WAIT, agent.reader.read::<EngineToAgent>())
            .await
            .expect("in time")
            .expect("reads")
            .expect("a message")
        {
            EngineToAgent::Job(found) => job = Some(found),
            EngineToAgent::DesiredState(state) => desired = Some(state),
        }
    }
    assert!(matches!(job, Some(Job::ResolveImage { image, .. }) if image == "nginx:1.29"));

    // A fake telemetry answers the query with one batch of lines, then ends. The engine keeps
    // its side open while the stream lives, and closes it once the client leaves.
    let telemetry = UnixListener::bind(paths.telemetry_query_socket()).expect("binds");
    let expected = environment.clone();
    let fake = tokio::spawn(async move {
        let (stream, _) = telemetry.accept().await.expect("accepts");
        let (read, write) = stream.into_split();
        let mut reader = FramedReader::new(read);
        let query = reader
            .read::<LogQuery>()
            .await
            .expect("reads")
            .expect("a query");
        assert_eq!(query.environment_id, expected);
        assert!(query.follow);
        let early = timeout(Duration::from_millis(300), reader.read::<LogQuery>()).await;
        assert!(
            early.is_err(),
            "the engine closed its side of a live stream"
        );
        let mut writer = FramedWriter::new(write);
        writer
            .write(&LogStreamFrame::Lines(vec![LogLineDto {
                id: "1".to_owned(),
                timestamp: "2026-10-31T12:34:56.789Z".to_owned(),
                service_id: "svc".to_owned(),
                level: LogLevel::Info,
                message: "GET / 200".to_owned(),
            }]))
            .await
            .expect("writes");
        writer.write(&LogStreamFrame::End).await.expect("writes");
        let gone = timeout(WAIT, reader.read::<LogQuery>())
            .await
            .expect("in time");
        assert!(
            matches!(gone, Ok(None)),
            "the engine closes once the client leaves"
        );
    });

    let mut stream = connect(&paths.api_socket()).await;
    let opened = stream
        .call(
            7,
            Some(&session),
            ApiCall::OpenLogStream {
                environment_id: environment.clone(),
                since_ms: None,
            },
        )
        .await;
    assert_eq!(opened.id, 7);
    assert_eq!(opened.result, Ok(ApiValue::LogStreamOpened));
    let lines = timeout(WAIT, stream.reader.read::<LogStreamFrame>())
        .await
        .expect("in time")
        .expect("reads");
    assert!(matches!(lines, Some(LogStreamFrame::Lines(lines)) if lines[0].message == "GET / 200"));
    let end = timeout(WAIT, stream.reader.read::<LogStreamFrame>())
        .await
        .expect("in time")
        .expect("reads");
    assert_eq!(end, Some(LogStreamFrame::End));
    drop(stream);
    fake.await.expect("the fake telemetry is done");

    // Without a session, the stream is one failure reply.
    let mut refused = connect(&paths.api_socket()).await;
    let reply = refused
        .call(
            8,
            None,
            ApiCall::OpenLogStream {
                environment_id: environment,
                since_ms: None,
            },
        )
        .await;
    assert!(matches!(reply.result, Err(failure) if failure.kind == FailureKind::Unauthorized));

    // The log reaches the bucket.
    let store = DirStore::new(paths.bucket_dir());
    timeout(WAIT, async {
        while store.list("log/").expect("lists").is_empty() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("a segment in time");

    engine.abort();
}

#[tokio::test]
async fn bucket_conflict_makes_the_engine_read_only() {
    let home = home();
    let paths = Paths::new(home.path().to_path_buf());
    let (engine, session) = start(&paths).await;

    // Once the log so far is shipped, another server's object takes the next key.
    let store = DirStore::new(paths.bucket_dir());
    let next = timeout(WAIT, async {
        loop {
            if let Some(key) = store.list("log/").expect("lists").last() {
                let bytes = store.get(key).expect("reads").expect("exists");
                let header_end = bytes.iter().position(|b| *b == b'\n').expect("a header");
                let header: serde_json::Value =
                    serde_json::from_slice(&bytes[..header_end]).expect("JSON");
                let first = header["first_seq"].as_u64().expect("first_seq");
                let count = header["count"].as_u64().expect("count");
                return first + count;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("a segment in time");
    store
        .put_if_absent(
            &segment_key(GENERATION, next),
            b"{\"segment\":1,\"generation\":1,\"first_seq\":0,\"count\":1}\n{}\n",
        )
        .expect("writes");

    let mut api = connect(&paths.api_socket()).await;
    let refused = timeout(WAIT, async {
        let mut id = 0;
        loop {
            id += 1;
            let reply = api
                .call(
                    id,
                    Some(&session),
                    ApiCall::CreateProject {
                        source: NewProjectSourceDto::Empty,
                    },
                )
                .await;
            if let Err(failure) = reply.result {
                return failure;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .expect("read-only in time");
    assert_eq!(refused.kind, FailureKind::Conflict);
    assert_eq!(refused.message, "Another server writes to this bucket.");
    // Reads still work, and the other server's object is untouched.
    let reply = api.call(1000, Some(&session), ApiCall::ListProjects).await;
    assert!(matches!(reply.result, Ok(ApiValue::Projects(_))));
    // A login link is a write: the admin connection closes without a reply.
    let mut admin = connect(&paths.admin_socket()).await;
    admin
        .writer
        .write(&AdminRequest::CreateLoginLink)
        .await
        .expect("writes");
    let ended = timeout(WAIT, admin.reader.read::<AdminReply>())
        .await
        .expect("closed in time");
    assert!(matches!(ended, Ok(None)), "{ended:?}");
    assert!(
        store
            .get(&segment_key(GENERATION, next))
            .expect("reads")
            .is_some_and(|bytes| bytes.ends_with(b"\n{}\n"))
    );

    engine.abort();
}

#[tokio::test]
async fn a_new_agent_connection_replaces_the_old_one() {
    let home = home();
    let paths = Paths::new(home.path().to_path_buf());
    let (engine, session) = start(&paths).await;
    let hello = AgentToEngine::Hello {
        version: "test".to_owned(),
        applied_position: None,
    };
    let mut old = connect(&paths.agent_socket()).await;
    old.writer.write(&hello).await.expect("writes");
    let first = timeout(WAIT, old.reader.read::<EngineToAgent>())
        .await
        .expect("in time");
    assert!(matches!(first, Ok(Some(EngineToAgent::DesiredState(_)))));

    let mut new = connect(&paths.agent_socket()).await;
    new.writer.write(&hello).await.expect("writes");
    let first = timeout(WAIT, new.reader.read::<EngineToAgent>())
        .await
        .expect("in time");
    assert!(matches!(first, Ok(Some(EngineToAgent::DesiredState(_)))));
    // The old connection leaving does not disconnect the new one.
    drop(old);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut api = connect(&paths.api_socket()).await;
    let reply = api
        .call(
            1,
            Some(&session),
            ApiCall::CreateProject {
                source: NewProjectSourceDto::Image {
                    image: "redis:8".to_owned(),
                },
            },
        )
        .await;
    assert!(reply.result.is_ok());
    let job = timeout(WAIT, async {
        loop {
            match new.reader.read::<EngineToAgent>().await {
                Ok(Some(EngineToAgent::Job(job))) => return job,
                Ok(Some(EngineToAgent::DesiredState(_))) => {}
                other => panic!("the agent connection ended: {other:?}"),
            }
        }
    })
    .await
    .expect("a job in time");
    assert!(matches!(job, Job::ResolveImage { image, .. } if image == "redis:8"));

    engine.abort();
}

/// Every regular file under `dir`.
fn files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).expect("reads the directory") {
        let entry = entry.expect("an entry");
        let kind = entry.file_type().expect("a file type");
        if kind.is_dir() {
            files.extend(files_under(&entry.path()));
        } else if kind.is_file() {
            files.push(entry.path());
        }
    }
    files
}

#[tokio::test]
async fn admin_socket_configures_the_ingress_and_seals_its_token() {
    let home = home();
    let paths = Paths::new(home.path().to_path_buf());
    let (engine, session) = start(&paths).await;
    let token = "eyJhIjoiYWNjb3VudCJ9-tunnel-token";

    // `brume install` sends the zone, the tunnel and its token on `admin.sock`.
    let mut admin = connect(&paths.admin_socket()).await;
    admin
        .writer
        .write(&AdminRequest::ConfigureIngress {
            zone: "example.com".to_owned(),
            tunnel_id: "c1744f8b-faa1-48a4-9e5c-02ac921467fa".to_owned(),
            tunnel_token: SecretString::from(token),
        })
        .await
        .expect("writes");
    let reply = timeout(WAIT, admin.reader.read::<AdminReply>())
        .await
        .expect("in time")
        .expect("reads");
    assert!(
        matches!(reply, Some(AdminReply::IngressConfigured)),
        "{reply:?}"
    );

    // The panel reads the zone.
    let mut api = connect(&paths.api_socket()).await;
    let reply = api.call(1, Some(&session), ApiCall::GetIngress).await;
    assert!(
        matches!(&reply.result, Ok(ApiValue::Ingress(Some(ingress))) if ingress.zone == "example.com"),
        "{reply:?}"
    );

    // No file of the home holds the token in clear: the database seals it.
    for file in files_under(home.path()) {
        let bytes = std::fs::read(&file).expect("reads");
        assert!(
            !bytes
                .windows(token.len())
                .any(|window| window == token.as_bytes()),
            "{}",
            file.display()
        );
    }

    engine.abort();
}
