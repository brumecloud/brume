//! The HTTP layer against a fake engine that replays fixed answers on a temporary `api.sock`.

use std::path::Path;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use brume_api::{ApiConfig, ApiFileConfig, app};
use brume_proto::api::{
    ApiCall, ApiFailure, ApiReply, ApiRequest, ApiValue, CreatedProjectDto, FailureKind, LogLevel,
    LogLineDto, ProjectDto,
};
use brume_proto::telemetry::LogStreamFrame;
use brume_runtime::framed::{FramedReader, FramedWriter};
use brume_runtime::paths::Paths;
use serde_json::{Value, json};
use tokio::net::UnixListener;
use tower::ServiceExt as _;

const HOST: &str = "127.0.0.1:4000";

const CSP: &str = "default-src 'self'; base-uri 'none'; form-action 'self'; \
                   frame-ancestors 'none'; require-trusted-types-for 'script'";

/// What the fake engine answers to a call.
type Answer = Box<dyn Fn(&ApiCall) -> Result<ApiValue, ApiFailure> + Send + Sync>;

/// A fake engine on `<home>/run/api.sock`. It records each request, answers with `answer`, and
/// streams `frames` after a `LogStreamOpened`.
struct FakeEngine {
    requests: Arc<Mutex<Vec<ApiRequest>>>,
}

impl FakeEngine {
    fn start(home: &Path, answer: Answer, frames: Vec<LogStreamFrame>) -> Self {
        Self::spawn(home, answer, frames, Connections::KeepOpen)
    }

    fn spawn(
        home: &Path,
        answer: Answer,
        frames: Vec<LogStreamFrame>,
        connections: Connections,
    ) -> Self {
        let run = home.join("run");
        std::fs::create_dir_all(&run).expect("creates the run directory");
        let listener = UnixListener::bind(run.join("api.sock")).expect("binds api.sock");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let answer = Arc::new(answer);
        let recorded = Arc::clone(&requests);
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.expect("accepts");
                let answer = Arc::clone(&answer);
                let recorded = Arc::clone(&recorded);
                let frames = frames.clone();
                tokio::spawn(async move {
                    let (read, write) = stream.into_split();
                    let mut reader = FramedReader::new(read);
                    let mut writer = FramedWriter::new(write);
                    while let Ok(Some(request)) = reader.read::<ApiRequest>().await {
                        recorded.lock().expect("lock").push(request.clone());
                        let result = answer(&request.body);
                        let opened = matches!(result, Ok(ApiValue::LogStreamOpened));
                        let reply = ApiReply {
                            id: request.id,
                            result,
                        };
                        if writer.write(&reply).await.is_err()
                            || connections == Connections::CloseAfterReply
                        {
                            return;
                        }
                        if opened {
                            for frame in &frames {
                                if writer.write(frame).await.is_err() {
                                    return;
                                }
                            }
                            return;
                        }
                    }
                });
            }
        });
        Self { requests }
    }

    fn requests(&self) -> Vec<ApiRequest> {
        self.requests.lock().expect("lock").clone()
    }
}

/// Whether the fake engine keeps a connection open after a reply.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Connections {
    KeepOpen,
    CloseAfterReply,
}

/// A home under `/tmp`: macOS limits a unix socket path to 104 bytes.
fn home() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("brume-api")
        .tempdir_in("/tmp")
        .expect("creates a temporary home")
}

fn router(home: &Path) -> Router {
    app(&Paths::new(home.to_path_buf()), &ApiConfig::new(4000))
}

fn project() -> ProjectDto {
    ProjectDto {
        id: "prj".to_owned(),
        name: "Shop".to_owned(),
        description: String::new(),
        updated_at: "2026-10-31T12:34:56.789Z".to_owned(),
    }
}

/// Answers every call with the empty list of projects, and logins with a session.
fn default_answer() -> Answer {
    Box::new(|call| match call {
        ApiCall::Login { .. } => Ok(ApiValue::Session {
            token: "sess-123".to_owned(),
            admin: "adm".to_owned(),
        }),
        ApiCall::Logout => Ok(ApiValue::NoContent),
        ApiCall::CreateProject { .. } => Ok(ApiValue::CreatedProject(CreatedProjectDto {
            project_id: "prj".to_owned(),
            environment_name: "production".to_owned(),
        })),
        ApiCall::OpenLogStream { .. } => Ok(ApiValue::LogStreamOpened),
        _ => Ok(ApiValue::Projects(vec![project()])),
    })
}

/// A request from the panel: same origin, with `X-Brume`.
fn panel(method: &str, uri: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::HOST, HOST)
        .header("sec-fetch-site", "same-origin")
        .header("x-brume", "1")
}

fn json_body(value: &Value) -> Body {
    Body::from(serde_json::to_vec(value).expect("serializes"))
}

async fn send(router: &Router, request: Request<Body>) -> Response<Body> {
    router.clone().oneshot(request).await.expect("answers")
}

async fn body_bytes(response: Response<Body>) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .expect("reads the body")
        .to_vec()
}

async fn body_json(response: Response<Body>) -> Value {
    serde_json::from_slice(&body_bytes(response).await).expect("a JSON body")
}

fn header_of<'a>(response: &'a Response<Body>, name: &str) -> Option<&'a str> {
    response.headers().get(name).and_then(|v| v.to_str().ok())
}

#[tokio::test]
async fn panel_request_reaches_the_engine() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = router(home.path());
    let response = send(
        &router,
        panel("GET", "/api/projects")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header_of(&response, "cache-control"), Some("no-store"));
    for (name, _) in response.headers() {
        assert!(!name.as_str().starts_with("access-control-"), "{name}");
    }
    assert_eq!(
        body_json(response).await,
        json!([{"id": "prj", "name": "Shop", "description": "", "updatedAt": "2026-10-31T12:34:56.789Z"}])
    );
    let requests = engine.requests();
    assert_eq!(requests.len(), 1);
    assert!(matches!(requests[0].body, ApiCall::ListProjects));
    assert_eq!(requests[0].session, None);
}

#[tokio::test]
async fn rejects_missing_x_brume() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let request = Request::builder()
        .uri("/api/projects")
        .header(header::HOST, HOST)
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .expect("request");
    let response = send(&router(home.path()), request).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(body_json(response).await["message"].is_string());
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn rejects_cross_site() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = router(home.path());
    for site in ["cross-site", "same-site", "none"] {
        let request = panel("GET", "/api/projects")
            .header("sec-fetch-site", site)
            .body(Body::empty())
            .expect("request");
        assert_eq!(send(&router, request).await.status(), StatusCode::FORBIDDEN);
    }
    // A link from another site cannot open the panel either; a typed address can.
    let request = Request::builder()
        .uri("/projects/x")
        .header(header::HOST, HOST)
        .header("sec-fetch-site", "cross-site")
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::FORBIDDEN);
    let request = Request::builder()
        .uri("/projects/x")
        .header(header::HOST, HOST)
        .header("sec-fetch-site", "none")
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::OK);
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn rejects_foreign_origin_on_post() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = router(home.path());
    let body = json!({"kind": "empty"});
    for origin in ["http://evil.example", "null", "https://127.0.0.1:4000"] {
        let request = panel("POST", "/api/projects")
            .header(header::ORIGIN, origin)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json_body(&body))
            .expect("request");
        assert_eq!(send(&router, request).await.status(), StatusCode::FORBIDDEN);
    }
    let request = panel("DELETE", "/api/services/svc")
        .header(header::ORIGIN, "http://evil.example")
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::FORBIDDEN);
    assert!(engine.requests().is_empty());

    for origin in ["http://127.0.0.1:4000", "http://localhost:5173"] {
        let request = panel("POST", "/api/projects")
            .header(header::ORIGIN, origin)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json_body(&body))
            .expect("request");
        let response = send(&router, request).await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            body_json(response).await,
            json!({"projectId": "prj", "environmentName": "production"})
        );
    }
}

#[tokio::test]
async fn rejects_a_body_that_is_not_json() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let request = panel("POST", "/api/projects")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(r#"{"kind": "empty"}"#))
        .expect("request");
    let response = send(&router(home.path()), request).await;
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let request = panel("POST", "/api/projects")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"kind": "s3cr3t"}"#))
        .expect("request");
    let response = send(&router(home.path()), request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_bytes(response).await;
    assert!(!String::from_utf8_lossy(&body).contains("s3cr3t"));
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn rejects_unknown_host() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = router(home.path());
    for host in ["evil.example:4000", "127.0.0.1:4001", "localhost"] {
        let request = panel("GET", "/api/projects")
            .header(header::HOST, host)
            .body(Body::empty())
            .expect("request");
        assert_eq!(
            send(&router, request).await.status(),
            StatusCode::MISDIRECTED_REQUEST
        );
    }
    let request = Request::builder()
        .uri("/")
        .header(header::HOST, "evil.example")
        .body(Body::empty())
        .expect("request");
    assert_eq!(
        send(&router, request).await.status(),
        StatusCode::MISDIRECTED_REQUEST
    );
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn rejects_oversized_body() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = router(home.path());
    let image = "x".repeat(64 * 1024);
    let body = json!({"kind": "image", "image": image});
    let request = panel("POST", "/api/projects")
        .header(header::CONTENT_TYPE, "application/json")
        .body(json_body(&body))
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(body_json(response).await["message"].is_string());

    // Without a length: the limit applies while reading.
    let chunks = vec![Ok::<_, std::io::Error>(
        serde_json::to_vec(&body).expect("json"),
    )];
    let request = panel("POST", "/api/projects")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from_stream(futures_util::stream::iter(chunks)))
        .expect("request");
    assert_eq!(
        send(&router, request).await.status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn login_sets_httponly_strict_cookie() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = router(home.path());
    let request = panel("POST", "/api/session")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ORIGIN, "http://127.0.0.1:4000")
        .body(json_body(&json!({"token": "login-link-token"})))
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        header_of(&response, "set-cookie"),
        Some("brume_session=sess-123; HttpOnly; SameSite=Strict; Path=/")
    );
    assert!(body_bytes(response).await.is_empty());

    let request = panel("GET", "/api/projects")
        .header(header::COOKIE, "theme=dark; brume_session=sess-123")
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::OK);

    let request = panel("DELETE", "/api/session")
        .header(header::COOKIE, "brume_session=sess-123")
        .body(Body::empty())
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        header_of(&response, "set-cookie"),
        Some("brume_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0")
    );

    let requests = engine.requests();
    assert_eq!(requests.len(), 3);
    assert!(matches!(&requests[0].body, ApiCall::Login { token } if token == "login-link-token"));
    assert_eq!(requests[0].session, None);
    assert!(matches!(requests[1].body, ApiCall::ListProjects));
    assert_eq!(requests[1].session.as_deref(), Some("sess-123"));
    assert!(matches!(requests[2].body, ApiCall::Logout));
    assert_eq!(requests[2].session.as_deref(), Some("sess-123"));
    // One multiplexed connection, with increasing ids.
    assert!(requests[0].id < requests[1].id && requests[1].id < requests[2].id);
}

#[tokio::test]
async fn maps_failure_kinds_to_status() {
    let home = home();
    let kinds = [
        (FailureKind::Unauthorized, StatusCode::UNAUTHORIZED),
        (FailureKind::NotFound, StatusCode::NOT_FOUND),
        (FailureKind::Invalid, StatusCode::BAD_REQUEST),
        (FailureKind::Conflict, StatusCode::CONFLICT),
        (FailureKind::Internal, StatusCode::INTERNAL_SERVER_ERROR),
    ];
    let answer: Answer = Box::new(|call| {
        let kind = match call {
            ApiCall::GetServiceSettings { service_id } => match service_id.as_str() {
                "unauthorized" => FailureKind::Unauthorized,
                "not-found" => FailureKind::NotFound,
                "invalid" => FailureKind::Invalid,
                "conflict" => FailureKind::Conflict,
                _ => FailureKind::Internal,
            },
            _ => FailureKind::Internal,
        };
        Err(ApiFailure {
            kind,
            message: format!("Failed: {kind:?}."),
        })
    });
    let _engine = FakeEngine::start(home.path(), answer, vec![]);
    let router = router(home.path());
    for ((kind, status), id) in kinds.into_iter().zip([
        "unauthorized",
        "not-found",
        "invalid",
        "conflict",
        "internal",
    ]) {
        let request = panel("GET", &format!("/api/services/{id}/settings"))
            .body(Body::empty())
            .expect("request");
        let response = send(&router, request).await;
        assert_eq!(response.status(), status);
        assert_eq!(
            body_json(response).await,
            json!({"message": format!("Failed: {kind:?}.")})
        );
    }
}

#[tokio::test]
async fn answers_unavailable_sources_and_routes() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = router(home.path());
    let request = panel("GET", "/api/github/repositories")
        .body(Body::empty())
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        body_json(response).await,
        json!({"message": "GitHub sources are not available yet."})
    );

    let request = panel("POST", "/api/projects")
        .header(header::CONTENT_TYPE, "application/json")
        .body(json_body(
            &json!({"kind": "github", "repository": "org/app"}),
        ))
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json(response).await,
        json!({"message": "github sources are not available yet."})
    );

    let request = panel("GET", "/api/nope")
        .body(Body::empty())
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(body_json(response).await["message"].is_string());
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn answers_503_without_an_engine() {
    let home = home();
    let request = panel("GET", "/api/projects")
        .body(Body::empty())
        .expect("request");
    let response = send(&router(home.path()), request).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(body_json(response).await["message"].is_string());
}

#[tokio::test]
async fn serves_spa_with_csp() {
    let home = home();
    let router = router(home.path());
    for uri in ["/", "/projects/x"] {
        let request = Request::builder()
            .uri(uri)
            .header(header::HOST, HOST)
            .body(Body::empty())
            .expect("request");
        let response = send(&router, request).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            header_of(&response, "content-type"),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(header_of(&response, "content-security-policy"), Some(CSP));
        assert_eq!(
            header_of(&response, "x-content-type-options"),
            Some("nosniff")
        );
        assert_eq!(header_of(&response, "referrer-policy"), Some("same-origin"));
        let body = String::from_utf8(body_bytes(response).await).expect("UTF-8");
        assert!(body.to_ascii_lowercase().contains("<html"), "{body}");
    }
}

fn line(id: &str, message: &str) -> LogLineDto {
    LogLineDto {
        id: id.to_owned(),
        timestamp: "2026-10-31T12:34:56.789Z".to_owned(),
        service_id: "svc".to_owned(),
        level: LogLevel::Info,
        message: message.to_owned(),
    }
}

#[tokio::test]
async fn sse_relays_frames() {
    let home = home();
    let frames = vec![
        LogStreamFrame::Lines(vec![line("1", "GET / 200"), line("2", "<script>")]),
        LogStreamFrame::Lines(vec![line("3", "done")]),
        LogStreamFrame::End,
    ];
    let engine = FakeEngine::start(home.path(), default_answer(), frames);
    // EventSource cannot send X-Brume: Fetch Metadata guards the stream.
    let request = Request::builder()
        .uri("/api/environments/env/logs")
        .header(header::HOST, HOST)
        .header("sec-fetch-site", "same-origin")
        .header(header::COOKIE, "brume_session=sess-123")
        .body(Body::empty())
        .expect("request");
    let response = send(&router(home.path()), request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header_of(&response, "content-type"),
        Some("text/event-stream")
    );
    let body = String::from_utf8(body_bytes(response).await).expect("UTF-8");
    let first = serde_json::to_string(&[line("1", "GET / 200"), line("2", "<script>")])
        .expect("serializes");
    let second = serde_json::to_string(&[line("3", "done")]).expect("serializes");
    assert_eq!(body, format!("data: {first}\n\ndata: {second}\n\n"));

    let requests = engine.requests();
    assert_eq!(requests.len(), 1);
    assert!(matches!(
        &requests[0].body,
        ApiCall::OpenLogStream { environment_id, since_ms: None } if environment_id == "env"
    ));
    assert_eq!(requests[0].session.as_deref(), Some("sess-123"));
}

#[tokio::test]
async fn sse_refused_by_the_engine_is_a_json_error() {
    let home = home();
    let answer: Answer = Box::new(|_| {
        Err(ApiFailure {
            kind: FailureKind::Unauthorized,
            message: "Log in again.".to_owned(),
        })
    });
    let _engine = FakeEngine::start(home.path(), answer, vec![]);
    let request = Request::builder()
        .uri("/api/environments/env/logs")
        .header(header::HOST, HOST)
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .expect("request");
    let response = send(&router(home.path()), request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        body_json(response).await,
        json!({"message": "Log in again."})
    );
}

#[tokio::test]
async fn logout_of_an_expired_session_still_clears_the_cookie() {
    let home = home();
    let answer: Answer = Box::new(|_| {
        Err(ApiFailure {
            kind: FailureKind::Unauthorized,
            message: "Log in again.".to_owned(),
        })
    });
    let _engine = FakeEngine::start(home.path(), answer, vec![]);
    let request = panel("DELETE", "/api/session")
        .header(header::COOKIE, "brume_session=expired")
        .body(Body::empty())
        .expect("request");
    let response = send(&router(home.path()), request).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        header_of(&response, "set-cookie"),
        Some("brume_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0")
    );
}

#[tokio::test]
async fn reconnects_after_the_engine_closes_the_connection() {
    let home = home();
    let engine = FakeEngine::spawn(
        home.path(),
        default_answer(),
        vec![],
        Connections::CloseAfterReply,
    );
    let router = router(home.path());
    let list = || {
        panel("GET", "/api/projects")
            .body(Body::empty())
            .expect("request")
    };
    assert_eq!(send(&router, list()).await.status(), StatusCode::OK);
    // A call can race the close and get a 503; the client then opens a new connection.
    let mut statuses = Vec::new();
    for _ in 0..5 {
        let status = send(&router, list()).await.status();
        statuses.push(status);
        if status == StatusCode::OK {
            break;
        }
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }
    assert_eq!(statuses.last(), Some(&StatusCode::OK), "{statuses:?}");
    assert!(engine.requests().len() >= 2);
}

/// The machine's name on the tailnet, as `tailscale serve` passes it in `Host`.
const TAILNET_HOST: &str = "brume.tail1234.ts.net";

/// The admin's Tailscale login, the only one in the allowlist.
const ADMIN_LOGIN: &str = "admin@example.com";

/// The API of a host behind `tailscale serve`, as `/etc/brume/api.toml` configures it.
fn tailnet_router(home: &Path) -> Router {
    let file = ApiFileConfig {
        port: 4000,
        hosts: vec![TAILNET_HOST.to_owned()],
        tailscale_logins: vec![ADMIN_LOGIN.to_owned()],
        public_url: format!("https://{TAILNET_HOST}"),
    };
    app(
        &Paths::new(home.to_path_buf()),
        &ApiConfig::from_file(&file),
    )
}

/// A request from the panel through `tailscale serve`, which sets the admin's login.
fn tailnet(method: &str, uri: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::HOST, TAILNET_HOST)
        .header("sec-fetch-site", "same-origin")
        .header("x-brume", "1")
        .header("tailscale-user-login", ADMIN_LOGIN)
}

/// The same request, without a `Tailscale-User-Login`.
fn tailnet_anonymous(method: &str, uri: &str, host: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::HOST, host)
        .header("sec-fetch-site", "same-origin")
        .header("x-brume", "1")
}

#[tokio::test]
async fn missing_tailscale_login_is_403() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = tailnet_router(home.path());
    // Through the tailnet, through an SSH tunnel, and on a route that does not exist.
    for (uri, host) in [
        ("/api/projects", TAILNET_HOST),
        ("/api/projects", "127.0.0.1:4000"),
        ("/api/nothing-here", TAILNET_HOST),
    ] {
        let request = tailnet_anonymous("GET", uri, host)
            .body(Body::empty())
            .expect("request");
        let response = send(&router, request).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri} on {host}");
        assert_eq!(header_of(&response, "content-security-policy"), Some(CSP));
        assert!(body_json(response).await["message"].is_string());
    }
    // An empty login is no login.
    let request = tailnet_anonymous("GET", "/api/projects", TAILNET_HOST)
        .header("tailscale-user-login", "")
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::FORBIDDEN);
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn unknown_tailscale_login_is_403() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = tailnet_router(home.path());
    for logins in [
        &["mallory@example.com"][..],
        &["Admin@example.com"],
        &["admin@example.com "],
        // `tailscale serve` sets one; a second one is forged.
        &[ADMIN_LOGIN, "mallory@example.com"],
        &[ADMIN_LOGIN, ADMIN_LOGIN],
    ] {
        let mut request = tailnet_anonymous("GET", "/api/projects", TAILNET_HOST);
        for login in logins {
            request = request.header("tailscale-user-login", *login);
        }
        let request = request.body(Body::empty()).expect("request");
        assert_eq!(
            send(&router, request).await.status(),
            StatusCode::FORBIDDEN,
            "{logins:?}"
        );
    }
    assert!(engine.requests().is_empty());
}

#[tokio::test]
async fn allowed_login_passes() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = tailnet_router(home.path());
    let request = tailnet("GET", "/api/projects")
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::OK);
    // Through an SSH tunnel too, with the header set by hand.
    let request = tailnet_anonymous("GET", "/api/projects", "localhost:4000")
        .header("tailscale-user-login", ADMIN_LOGIN)
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::OK);
    assert_eq!(engine.requests().len(), 2);
}

#[tokio::test]
async fn tailnet_host_accepted() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let dev = router(home.path());
    let router = tailnet_router(home.path());
    for host in [
        TAILNET_HOST,
        "brume.tail1234.ts.net:443",
        "BRUME.tail1234.ts.net",
    ] {
        let request = tailnet_anonymous("GET", "/api/projects", host)
            .header("tailscale-user-login", ADMIN_LOGIN)
            .body(Body::empty())
            .expect("request");
        assert_eq!(
            send(&router, request).await.status(),
            StatusCode::OK,
            "{host}"
        );
    }
    // The Host check comes first: 421 even with the admin's login.
    for host in [
        "other.tail1234.ts.net",
        "brume.tail1234.ts.net:80",
        "brume.tail1234.ts.net.evil.example",
        "127.0.0.1:5173",
    ] {
        let request = tailnet_anonymous("GET", "/api/projects", host)
            .header("tailscale-user-login", ADMIN_LOGIN)
            .body(Body::empty())
            .expect("request");
        assert_eq!(
            send(&router, request).await.status(),
            StatusCode::MISDIRECTED_REQUEST,
            "{host}"
        );
    }
    // `brume dev` does not serve the tailnet name.
    let request = tailnet_anonymous("GET", "/api/projects", TAILNET_HOST)
        .body(Body::empty())
        .expect("request");
    assert_eq!(
        send(&dev, request).await.status(),
        StatusCode::MISDIRECTED_REQUEST
    );
    assert_eq!(engine.requests().len(), 3);
}

#[tokio::test]
async fn https_origin_of_tailnet_host_accepted_on_write() {
    let home = home();
    let engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = tailnet_router(home.path());
    let body = json!({"kind": "empty"});
    for origin in [
        "http://brume.tail1234.ts.net",
        "https://brume.tail1234.ts.net.evil.example",
        "https://other.tail1234.ts.net",
        "http://localhost:5173",
    ] {
        let request = tailnet("POST", "/api/projects")
            .header(header::ORIGIN, origin)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json_body(&body))
            .expect("request");
        assert_eq!(
            send(&router, request).await.status(),
            StatusCode::FORBIDDEN,
            "{origin}"
        );
    }
    assert!(engine.requests().is_empty());
    for origin in ["https://brume.tail1234.ts.net", "http://127.0.0.1:4000"] {
        let request = tailnet("POST", "/api/projects")
            .header(header::ORIGIN, origin)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json_body(&body))
            .expect("request");
        assert_eq!(
            send(&router, request).await.status(),
            StatusCode::CREATED,
            "{origin}"
        );
    }
}

#[tokio::test]
async fn cookie_is_secure_with_https_public_url() {
    let home = home();
    let _engine = FakeEngine::start(home.path(), default_answer(), vec![]);
    let router = tailnet_router(home.path());
    let request = tailnet("POST", "/api/session")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ORIGIN, "https://brume.tail1234.ts.net")
        .body(json_body(&json!({"token": "login-link-token"})))
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        header_of(&response, "set-cookie"),
        Some("brume_session=sess-123; HttpOnly; SameSite=Strict; Path=/; Secure")
    );
    let request = tailnet("DELETE", "/api/session")
        .header(header::COOKIE, "brume_session=sess-123")
        .body(Body::empty())
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(
        header_of(&response, "set-cookie"),
        Some("brume_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0; Secure")
    );

    // Through an SSH tunnel, the panel is plain HTTP: no `Secure`.
    let file = ApiFileConfig {
        port: 4000,
        hosts: vec![],
        tailscale_logins: vec![],
        public_url: String::new(),
    };
    let tunnel = app(
        &Paths::new(home.path().to_path_buf()),
        &ApiConfig::from_file(&file),
    );
    let request = panel("POST", "/api/session")
        .header(header::CONTENT_TYPE, "application/json")
        .body(json_body(&json!({"token": "login-link-token"})))
        .expect("request");
    let response = send(&tunnel, request).await;
    assert_eq!(
        header_of(&response, "set-cookie"),
        Some("brume_session=sess-123; HttpOnly; SameSite=Strict; Path=/")
    );
}

#[tokio::test]
async fn static_panel_also_requires_login() {
    let home = home();
    let router = tailnet_router(home.path());
    let typed = |host: &str| {
        Request::builder()
            .uri("/projects/x")
            .header(header::HOST, host)
            .header("sec-fetch-site", "none")
    };
    let request = typed(TAILNET_HOST).body(Body::empty()).expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(header_of(&response, "content-security-policy"), Some(CSP));
    let request = typed(TAILNET_HOST)
        .header("tailscale-user-login", "mallory@example.com")
        .body(Body::empty())
        .expect("request");
    assert_eq!(send(&router, request).await.status(), StatusCode::FORBIDDEN);
    let request = typed("evil.example")
        .header("tailscale-user-login", ADMIN_LOGIN)
        .body(Body::empty())
        .expect("request");
    assert_eq!(
        send(&router, request).await.status(),
        StatusCode::MISDIRECTED_REQUEST
    );
    let request = typed(TAILNET_HOST)
        .header("tailscale-user-login", ADMIN_LOGIN)
        .body(Body::empty())
        .expect("request");
    let response = send(&router, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = String::from_utf8(body_bytes(response).await).expect("UTF-8");
    assert!(body.to_ascii_lowercase().contains("<html"), "{body}");
}
