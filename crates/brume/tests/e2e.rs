//! `brume dev` end to end, against the real Docker of this machine: the binary runs every
//! service, and the test speaks only HTTP to the API, as the panel does, then checks Docker and
//! the bucket. Ignored by default: run with
//! `cargo test -p brume --test e2e -- --ignored --test-threads=1`.
//!
//! The agent removes every `brume.managed` container that its engine does not want, so the
//! test refuses to run while another Brume runs containers on this Docker.
//!
//! The scenario ends with the public entry: a fake tunnel token, so cloudflared fails, and
//! Traefik routes a service's domain, which the test checks from the ingress network.

use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateImageOptionsBuilder, InspectContainerOptions, ListContainersOptionsBuilder,
    ListNetworksOptionsBuilder, ListVolumesOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder, RemoveVolumeOptions, StartContainerOptions,
    WaitContainerOptions,
};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_proto::frame::{FrameDecoder, encode};
use futures_util::StreamExt as _;
use reqwest::header::{COOKIE, SET_COOKIE};
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};

const IMAGE: &str = "nginx:1.29-alpine";
const MISSING_IMAGE: &str = "nginx:0.0.0-brume-e2e-missing";
const GREETING: &str = "a\nb=\"c\" 🚀";
const TOKEN: &str = "s3cr3t-🚀";
/// A tunnel token that Cloudflare refuses: cloudflared fails, and the rest goes on.
const TUNNEL_TOKEN: &str = "fake-tunnel-token-s3cr3t";
const ZONE: &str = "example.test";
/// The image that checks the entry from inside Docker.
const CURL_IMAGE: &str = "curlimages/curl:8.17.0";
/// Traefik's entrypoint, on the ingress network.
const TRAEFIK_URL: &str = "http://10.213.255.10:8080/";

/// The longest wait for a deployment, a log line or a container.
const WAIT: Duration = Duration::from_secs(60);
/// The longest wait for `brume dev` to print its login link.
const STARTUP: Duration = Duration::from_secs(60);
const POLL: Duration = Duration::from_millis(250);

/// A `brume dev` process, its home and its log. Dropping it stops it.
struct Dev {
    child: Child,
    dir: tempfile::TempDir,
    port: u16,
    token: String,
    /// The pids of the services, as `brume dev` printed them.
    pids: Vec<u32>,
}

impl Dev {
    /// Starts `brume dev` on a free port with `home` under a temporary directory, and waits for
    /// its login link.
    ///
    /// The directory is in `/tmp`, whose paths are short enough for a unix socket (104 bytes on
    /// macOS), but in `~/.cache` on macOS: OrbStack forwards no file event from `/tmp` to a
    /// container, and Traefik must see its config change.
    fn start(envs: &[(&str, &Path)], extra: &[&str], stdin: Option<&str>) -> Self {
        let mut base = PathBuf::from("/tmp");
        if cfg!(target_os = "macos")
            && let Some(home) = std::env::var_os("HOME")
        {
            base = PathBuf::from(home).join(".cache");
            std::fs::create_dir_all(&base).expect("creates ~/.cache");
        }
        let dir = tempfile::Builder::new()
            .prefix("brume-e2e")
            .tempdir_in(base)
            .expect("a temporary directory");
        Self::start_in(dir, envs, extra, stdin)
    }

    fn start_in(
        dir: tempfile::TempDir,
        envs: &[(&str, &Path)],
        extra: &[&str],
        stdin: Option<&str>,
    ) -> Self {
        let port = free_port();
        let log = std::fs::File::create(dir.path().join("dev.log")).expect("creates the log");
        let mut command = Command::new(env!("CARGO_BIN_EXE_brume"));
        command
            .arg("dev")
            .arg("--home")
            .arg(dir.path().join("home"))
            .arg("--port")
            .arg(port.to_string())
            .args(extra)
            .env("RUST_LOG", "debug")
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(log);
        for (key, value) in envs {
            command.env(key, value);
        }
        let mut child = command.spawn().expect("starts brume dev");
        if let Some(input) = stdin {
            let mut pipe = child.stdin.take().expect("a stdin");
            pipe.write_all(input.as_bytes()).expect("answers");
        }
        let stdout = child.stdout.take().expect("a stdout");
        let (lines, received) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { return };
                // Drains the pipe to the end, even once the test stops listening.
                let _ = lines.send(line);
            }
        });
        let mut dev = Self {
            child,
            dir,
            port,
            token: String::new(),
            pids: Vec::new(),
        };
        let deadline = Instant::now() + STARTUP;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let line = match received.recv_timeout(left) {
                Ok(line) => line,
                Err(error) => panic!("no login link ({error}); log:\n{}", dev.log()),
            };
            if let Some(token) = line
                .split_once("/login#token=")
                .map(|(_, token)| token.trim().to_owned())
            {
                dev.token = token;
            } else if let Some(pid) = line.split_once(" pid ").map(|(_, pid)| pid.trim()) {
                dev.pids.push(pid.parse().expect("a pid"));
            } else if line.trim() == "Press Ctrl-C to stop." {
                break;
            }
        }
        assert!(!dev.token.is_empty(), "no token; log:\n{}", dev.log());
        assert_eq!(dev.pids.len(), 4, "one pid per service");
        dev
    }

    fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    fn log(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("dev.log")).unwrap_or_default()
    }

    /// Sends Ctrl-C and waits for `brume dev` to exit.
    fn interrupt(&mut self) -> ExitStatus {
        self.stop().expect("brume dev stops on Ctrl-C")
    }

    /// Sends Ctrl-C, and its exit status unless it is still running 20 s later.
    fn stop(&mut self) -> Option<ExitStatus> {
        signal("-INT", self.child.id());
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Some(status);
            }
            std::thread::sleep(POLL);
        }
        None
    }
}

impl Drop for Dev {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait()
            && self.stop().is_none()
        {
            let _ = self.child.kill();
            let _ = self.child.wait();
            for pid in &self.pids {
                signal("-KILL", *pid);
            }
        }
    }
}

/// Runs `kill <signal> <pid>`. Whether the process got it.
fn signal(signal: &str, pid: u32) -> bool {
    Command::new("kill")
        .arg(signal)
        .arg(pid.to_string())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("binds")
        .local_addr()
        .expect("an address")
        .port()
}

/// The API, as the panel calls it: `X-Brume: 1` and the session cookie.
struct Api {
    http: reqwest::Client,
    base: String,
    cookie: String,
}

impl Api {
    /// Logs in with the link's token.
    async fn login(dev: &Dev) -> Self {
        let http = reqwest::Client::new();
        let base = format!("http://127.0.0.1:{}", dev.port);
        let response = http
            .post(format!("{base}/api/session"))
            .header("x-brume", "1")
            .json(&json!({ "token": dev.token }))
            .send()
            .await
            .expect("logs in");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let cookie = response
            .headers()
            .get(SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .expect("a session cookie")
            .to_owned();
        Self { http, base, cookie }
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, format!("{}{path}", self.base))
            .header("x-brume", "1")
            .header(COOKIE, &self.cookie)
    }

    /// Calls the API and checks the status. The body as JSON, `null` when empty.
    async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        expected: StatusCode,
    ) -> Value {
        let mut request = self.request(method.clone(), path);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.expect("calls the API");
        let status = response.status();
        let text = response.text().await.expect("a body");
        assert_eq!(status, expected, "{method} {path}: {text}");
        if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).expect("JSON")
        }
    }

    async fn get(&self, path: &str) -> Value {
        self.call(Method::GET, path, None, StatusCode::OK).await
    }

    async fn deployments(&self, service_id: &str) -> Vec<Value> {
        let all = self.get("/api/deployments").await;
        all.as_array()
            .expect("an array")
            .iter()
            .filter(|deployment| deployment["serviceId"] == service_id)
            .cloned()
            .collect()
    }

    async fn deployment(&self, deployment_id: &str) -> Option<Value> {
        let all = self.get("/api/deployments").await;
        all.as_array()
            .expect("an array")
            .iter()
            .find(|deployment| deployment["id"] == deployment_id)
            .cloned()
    }

    async fn service(&self, service_id: &str) -> Option<Value> {
        let all = self.get("/api/services").await;
        all.as_array()
            .expect("an array")
            .iter()
            .find(|service| service["id"] == service_id)
            .cloned()
    }

    /// Waits for the deployment to reach `status`, and returns it.
    async fn wait_status(&self, deployment_id: &str, status: &str) -> Value {
        let deadline = Instant::now() + WAIT;
        loop {
            let deployment = self.deployment(deployment_id).await;
            if let Some(deployment) = &deployment
                && deployment["status"] == status
            {
                return deployment.clone();
            }
            assert!(
                Instant::now() < deadline,
                "deployment {deployment_id} never became {status}: {deployment:?}"
            );
            tokio::time::sleep(POLL).await;
        }
    }

    /// Waits for the service to be `online`.
    async fn wait_online(&self, service_id: &str) {
        let deadline = Instant::now() + WAIT;
        loop {
            let service = self.service(service_id).await;
            if service
                .as_ref()
                .is_some_and(|service| service["status"] == "online")
            {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "service never online: {service:?}"
            );
            tokio::time::sleep(POLL).await;
        }
    }

    /// The newest deployment of the service other than `known`.
    async fn wait_new_deployment(&self, service_id: &str, known: &[String]) -> String {
        let deadline = Instant::now() + WAIT;
        loop {
            let fresh = self
                .deployments(service_id)
                .await
                .into_iter()
                .filter_map(|deployment| deployment["id"].as_str().map(str::to_owned))
                .find(|id| !known.contains(id));
            if let Some(id) = fresh {
                return id;
            }
            assert!(Instant::now() < deadline, "no new deployment");
            tokio::time::sleep(POLL).await;
        }
    }
}

/// Docker, as the test checks it.
async fn docker() -> Docker {
    Docker::connect_with_local_defaults()
        .expect("a Docker client")
        .negotiate_version()
        .await
        .expect("Docker answers")
}

/// The containers that carry `label`, stopped ones included.
async fn containers_with(docker: &Docker, label: &str) -> Vec<String> {
    let filters = HashMap::from([("label", vec![label.to_owned()])]);
    let options = ListContainersOptionsBuilder::new()
        .all(true)
        .filters(&filters)
        .build();
    docker
        .list_containers(Some(options))
        .await
        .expect("lists containers")
        .into_iter()
        .filter_map(|summary| summary.id)
        .collect()
}

/// The environment and running state of the one container of a deployment.
async fn container_of(docker: &Docker, deployment_id: &str) -> (Vec<String>, bool) {
    let ids = containers_with(docker, &format!("brume.deployment={deployment_id}")).await;
    assert_eq!(ids.len(), 1, "one container for {deployment_id}");
    let inspect = docker
        .inspect_container(&ids[0], None::<InspectContainerOptions>)
        .await
        .expect("inspects");
    let env = inspect
        .config
        .and_then(|config| config.env)
        .unwrap_or_default();
    let running = inspect
        .state
        .and_then(|state| state.running)
        .unwrap_or(false);
    (env, running)
}

/// The values of `name` in a container's environment.
fn values<'a>(env: &'a [String], name: &str) -> Vec<&'a str> {
    env.iter()
        .filter_map(|entry| entry.split_once('='))
        .filter(|(key, _)| *key == name)
        .map(|(_, value)| value)
        .collect()
}

/// Reads the log stream of the environment until a line of the service comes from nginx
/// itself, not from the image's entrypoint: its `nginx/1.x` notice.
async fn wait_for_nginx_line(api: &Api, environment_id: &str, service_id: &str) -> String {
    let response = api
        .http
        .get(format!(
            "{}/api/environments/{environment_id}/logs",
            api.base
        ))
        .header(COOKIE, &api.cookie)
        .send()
        .await
        .expect("opens the log stream");
    assert_eq!(response.status(), StatusCode::OK);
    let mut body = response.bytes_stream();
    let mut buffer = String::new();
    let read = async {
        while let Some(chunk) = body.next().await {
            buffer.push_str(&String::from_utf8_lossy(&chunk.expect("a chunk")));
            while let Some(end) = buffer.find("\n\n") {
                let event: String = buffer.drain(..end + 2).collect();
                for data in event.lines().filter_map(|line| line.strip_prefix("data: ")) {
                    let lines: Vec<Value> = serde_json::from_str(data).expect("LogLine[]");
                    let found = lines.iter().find(|line| {
                        line["serviceId"] == service_id
                            && line["message"]
                                .as_str()
                                .is_some_and(|message| message.contains("nginx/1."))
                    });
                    if let Some(line) = found {
                        return line["message"].as_str().unwrap_or_default().to_owned();
                    }
                }
            }
        }
        panic!("the log stream ended");
    };
    tokio::time::timeout(WAIT, read)
        .await
        .expect("a line from nginx in time")
}

/// The keys of the bucket's log segments, in order.
fn segments(home: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            // A segment being written is a temporary file whose name starts with a dot.
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                walk(&path, files);
            } else {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(&home.join("bucket/log"), &mut files);
    files.sort();
    files
}

/// Glues the segments back together and checks the hash chain from the first event. The
/// sequence number of the last event.
fn check_chain(files: &[PathBuf]) -> u64 {
    let mut previous = [0u8; 32];
    let mut next_seq = 1u64;
    for file in files {
        let bytes = std::fs::read(file).expect("reads a segment");
        let mut lines = bytes.split(|byte| *byte == b'\n');
        let header: Value =
            serde_json::from_slice(lines.next().expect("a header")).expect("a JSON header");
        assert_eq!(header["first_seq"], next_seq, "{}", file.display());
        let count = header["count"].as_u64().expect("a count");
        for _ in 0..count {
            let line = lines.next().expect("an event line");
            let event: Value = serde_json::from_slice(line).expect("a JSON event");
            assert_eq!(event["seq"], next_seq);
            assert_eq!(
                event["prev_hash"],
                hex::encode(previous),
                "event {next_seq}"
            );
            let mut hasher = Sha256::new();
            hasher.update(previous);
            hasher.update(line);
            previous = hasher.finalize().into();
            next_seq += 1;
        }
        assert_eq!(
            lines.next(),
            Some(&b""[..]),
            "a newline after the last event"
        );
        assert_eq!(lines.next(), None, "nothing after the last event");
    }
    next_seq - 1
}

/// Removes what the scenario left in Docker: the containers of Brume and of the test, the
/// networks of the entry, and the networks and volumes of its environments and services.
async fn clean_docker(environments: &[String], services: &[String]) {
    let docker = docker().await;
    let remove = RemoveContainerOptionsBuilder::new().force(true).build();
    for label in ["brume.managed=1", "brume.probe=1", "brume.e2e=1"] {
        for id in containers_with(&docker, label).await {
            let _ = docker.remove_container(&id, Some(remove.clone())).await;
        }
    }
    for network in ["brume-ingress", "brume-tunnel"] {
        let _ = docker.remove_network(network).await;
    }
    for environment in environments {
        let filters = HashMap::from([("label", vec![format!("brume.environment={environment}")])]);
        let networks = docker
            .list_networks(Some(
                ListNetworksOptionsBuilder::new().filters(&filters).build(),
            ))
            .await
            .unwrap_or_default();
        for network in networks.into_iter().filter_map(|network| network.name) {
            let _ = docker.remove_network(&network).await;
        }
    }
    for service in services {
        let filters = HashMap::from([("label", vec![format!("brume.service={service}")])]);
        let volumes = docker
            .list_volumes(Some(
                ListVolumesOptionsBuilder::new().filters(&filters).build(),
            ))
            .await
            .map(|list| list.volumes.unwrap_or_default())
            .unwrap_or_default();
        for volume in volumes {
            let _ = docker
                .remove_volume(&volume.name, None::<RemoveVolumeOptions>)
                .await;
        }
    }
}

/// Sets up the public entry on `admin.sock`, as `brume install --domain` does.
async fn configure_ingress(home: &Path) {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    let mut stream = tokio::net::UnixStream::connect(home.join("run/admin.sock"))
        .await
        .expect("connects to admin.sock");
    let request = AdminRequest::ConfigureIngress {
        zone: ZONE.to_owned(),
        tunnel_id: "c1744f8b-faa1-48a4-9e5c-02ac921467fa".to_owned(),
        tunnel_token: TUNNEL_TOKEN.into(),
    };
    stream
        .write_all(&encode(&request).expect("encodes"))
        .await
        .expect("sends");
    let mut decoder = FrameDecoder::default();
    let mut chunk = [0; 1024];
    let reply = loop {
        if let Some(reply) = decoder.next::<AdminReply>().expect("decodes") {
            break reply;
        }
        let read = tokio::time::timeout(WAIT, stream.read(&mut chunk))
            .await
            .expect("an answer in time")
            .expect("reads");
        assert!(read > 0, "the engine refused ConfigureIngress");
        decoder.push(&chunk[..read]);
    };
    assert!(reply == AdminReply::IngressConfigured, "unexpected reply");
}

/// Runs curl with `args` in a throwaway container on `network`: its exit code and its output.
async fn curl(docker: &Docker, network: &str, args: &[&str]) -> (i64, String) {
    let pull = CreateImageOptionsBuilder::new()
        .from_image(CURL_IMAGE)
        .build();
    let mut progress = docker.create_image(Some(pull), None, None);
    while let Some(item) = progress.next().await {
        item.expect("pulls curl");
    }
    let body = ContainerCreateBody {
        image: Some(CURL_IMAGE.to_owned()),
        cmd: Some(args.iter().map(|arg| (*arg).to_owned()).collect()),
        labels: Some(HashMap::from([("brume.e2e".to_owned(), "1".to_owned())])),
        host_config: Some(HostConfig {
            network_mode: Some(network.to_owned()),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    };
    let id = docker
        .create_container(
            None::<bollard::query_parameters::CreateContainerOptions>,
            body,
        )
        .await
        .expect("creates curl")
        .id;
    docker
        .start_container(&id, None::<StartContainerOptions>)
        .await
        .expect("starts curl");
    // bollard reports a non-zero exit code as an error.
    let code = match docker
        .wait_container(&id, None::<WaitContainerOptions>)
        .next()
        .await
    {
        Some(Ok(response)) => response.status_code,
        Some(Err(bollard::errors::Error::DockerContainerWaitError { code, .. })) => code,
        other => panic!("curl did not finish: {other:?}"),
    };
    let options = LogsOptionsBuilder::new().stdout(true).build();
    let mut logs = docker.logs(&id, Some(options));
    let mut output = String::new();
    while let Some(chunk) = logs.next().await {
        output.push_str(&chunk.expect("a log chunk").to_string());
    }
    let remove = RemoveContainerOptionsBuilder::new().force(true).build();
    let _ = docker.remove_container(&id, Some(remove)).await;
    (code, output)
}

/// The HTTP status and body that Traefik answers for `host`, from the ingress network.
async fn through_traefik(docker: &Docker, host: &str) -> (String, String) {
    let header = format!("Host: {host}");
    let (_, output) = curl(
        docker,
        "brume-ingress",
        &[
            "-s",
            "--max-time",
            "5",
            "-H",
            &header,
            "-w",
            "\n%{http_code}",
            TRAEFIK_URL,
        ],
    )
    .await;
    let (body, status) = output.rsplit_once('\n').unwrap_or(("", &output));
    (status.trim().to_owned(), body.to_owned())
}

/// Waits until Traefik answers `status` for `host`, and returns the body.
async fn wait_traefik(docker: &Docker, host: &str, status: &str) -> String {
    let deadline = Instant::now() + WAIT;
    loop {
        let (actual, body) = through_traefik(docker, host).await;
        if actual == status {
            return body;
        }
        assert!(
            Instant::now() < deadline,
            "Traefik never answered {status} for {host}: {actual} {body}"
        );
        tokio::time::sleep(POLL).await;
    }
}

/// What the scenario created, for the cleanup.
#[derive(Default)]
struct Created {
    environments: Vec<String>,
    services: Vec<String>,
}

#[tokio::test]
#[ignore = "needs Docker and the network: cargo test -p brume --test e2e -- --ignored"]
async fn e2e_deploy_nginx_from_the_api() {
    let docker = docker().await;
    let others = containers_with(&docker, "brume.managed=1").await;
    assert!(
        others.is_empty(),
        "another Brume runs containers on this Docker, which this test's agent would remove: \
         {others:?}"
    );

    let dev = Dev::start(&[], &[], None);
    let created = std::sync::Arc::new(std::sync::Mutex::new(Created::default()));
    // A panic in the scenario still stops brume dev and cleans Docker.
    let scenario = tokio::spawn(scenario(
        Api::login(&dev).await,
        dev.home(),
        std::sync::Arc::clone(&created),
    ));
    let outcome = scenario.await;
    let mut dev = dev;
    let status = dev.stop();
    let log = dev.log();
    let pids = dev.pids.clone();
    let home = dev.home();
    let created = std::mem::take(&mut *created.lock().expect("lock"));
    clean_docker(&created.environments, &created.services).await;
    if let Err(error) = outcome {
        eprintln!("brume dev log:\n{log}");
        std::panic::resume_unwind(error.into_panic());
    }

    let status = status.expect("brume dev stops on Ctrl-C");
    assert!(status.success(), "brume dev exited with {status}:\n{log}");
    for pid in pids {
        assert!(!signal("-0", pid), "service {pid} still runs");
    }
    // Even at `RUST_LOG=debug`, where bollard would print the container's environment.
    assert!(!log.contains("s3cr3t"), "a secret in the services' logs");
    assert!(log.contains("engine    "), "lines prefixed by the service");

    // A reset deletes the home, after confirmation.
    let mut reset = Command::new(env!("CARGO_BIN_EXE_brume"))
        .args([
            "dev",
            "--reset",
            "--port",
            &free_port().to_string(),
            "--home",
        ])
        .arg(&home)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("starts brume dev --reset");
    reset
        .stdin
        .take()
        .expect("a stdin")
        .write_all(b"n\n")
        .expect("answers");
    let refused = reset.wait().expect("waits");
    assert!(!refused.success(), "a refused reset stops brume dev");
    assert!(
        home.join("engine").exists(),
        "nothing deleted without a yes"
    );
}

/// Steps 1 to 10, over HTTP.
async fn scenario(api: Api, home: PathBuf, created: std::sync::Arc<std::sync::Mutex<Created>>) {
    let docker = docker().await;

    // 1. The link logged us in.
    let session = api.get("/api/session").await;
    assert!(session["admin"].is_string(), "{session}");
    eprintln!("step 1: logged in by the link");

    // 2. A project from an image.
    let project = api
        .call(
            Method::POST,
            "/api/projects",
            Some(json!({ "kind": "image", "image": IMAGE })),
            StatusCode::CREATED,
        )
        .await;
    let project_id = project["projectId"]
        .as_str()
        .expect("a project id")
        .to_owned();
    let environments = api.get("/api/environments").await;
    let environment_id = environments
        .as_array()
        .expect("an array")
        .iter()
        .find(|environment| environment["projectId"] == project_id.as_str())
        .and_then(|environment| environment["id"].as_str())
        .expect("the project's environment")
        .to_owned();
    created
        .lock()
        .expect("lock")
        .environments
        .push(environment_id.clone());
    let services = api.get("/api/services").await;
    let service_id = services
        .as_array()
        .expect("an array")
        .iter()
        .find(|service| service["environmentId"] == environment_id.as_str())
        .and_then(|service| service["id"].as_str())
        .expect("the image's service")
        .to_owned();
    created
        .lock()
        .expect("lock")
        .services
        .push(service_id.clone());
    eprintln!("step 2: project {project_id}, service {service_id}");

    // 3. Deployed and online.
    let first = api.wait_new_deployment(&service_id, &[]).await;
    api.wait_status(&first, "active").await;
    api.wait_online(&service_id).await;
    eprintln!("step 3: deployment {first} active, service online");

    // 4. nginx in the live logs.
    let line = wait_for_nginx_line(&api, &environment_id, &service_id).await;
    eprintln!("step 4: log line {line:?}");

    // 5. A variable and a secret, staged then deployed.
    for (name, kind, value) in [("GREETING", "plain", GREETING), ("TOKEN", "secret", TOKEN)] {
        api.call(
            Method::POST,
            "/api/changes",
            Some(json!({
                "serviceId": service_id,
                "type": "variable",
                "name": name,
                "kind": kind,
                "value": value,
            })),
            StatusCode::CREATED,
        )
        .await;
    }
    let changes = api.get("/api/changes").await;
    assert!(
        !changes.to_string().contains("s3cr3t"),
        "a staged secret is never sent back: {changes}"
    );
    let deployed = api
        .call(
            Method::POST,
            &format!("/api/environments/{environment_id}/changes/deploy"),
            None,
            StatusCode::OK,
        )
        .await;
    // The number of changes deployed.
    assert_eq!(deployed["deployed"], 2, "{deployed}");
    let second = api
        .wait_new_deployment(&service_id, std::slice::from_ref(&first))
        .await;
    api.wait_status(&second, "active").await;
    eprintln!("step 5: deployment {second} active with the variables");

    // 6. Docker has exactly these values.
    let (env, running) = container_of(&docker, &second).await;
    assert!(running);
    assert_eq!(values(&env, "GREETING"), [GREETING]);
    assert_eq!(values(&env, "TOKEN"), [TOKEN]);
    eprintln!("step 6: the container's environment holds both values exactly");

    // 7. Rollback to the first deployment: the variable is gone.
    let rollback = api
        .call(
            Method::POST,
            &format!("/api/deployments/{first}/rollback"),
            None,
            StatusCode::CREATED,
        )
        .await;
    let third = rollback["id"].as_str().expect("a deployment id").to_owned();
    api.wait_status(&third, "active").await;
    let (env, running) = container_of(&docker, &third).await;
    assert!(running);
    assert!(values(&env, "GREETING").is_empty(), "{env:?}");
    eprintln!(
        "step 7: rollback {third} active, GREETING gone (TOKEN: {} value(s))",
        values(&env, "TOKEN").len()
    );

    // 8. An image that does not exist: failed, and the running container stays.
    api.call(
        Method::POST,
        "/api/changes",
        Some(json!({
            "serviceId": service_id,
            "type": "setting",
            "field": "image",
            "value": MISSING_IMAGE,
        })),
        StatusCode::CREATED,
    )
    .await;
    api.call(
        Method::POST,
        &format!("/api/environments/{environment_id}/changes/deploy"),
        None,
        StatusCode::OK,
    )
    .await;
    let known = [first.clone(), second.clone(), third.clone()];
    let fourth = api.wait_new_deployment(&service_id, &known).await;
    api.wait_status(&fourth, "failed").await;
    let (_, running) = container_of(&docker, &third).await;
    assert!(running, "the previous container still runs");
    assert!(
        containers_with(&docker, &format!("brume.deployment={fourth}"))
            .await
            .is_empty()
    );
    eprintln!("step 8: deployment {fourth} failed, {third} still runs");

    // 9. The service goes, and its container with it.
    api.call(
        Method::DELETE,
        &format!("/api/services/{service_id}"),
        None,
        StatusCode::NO_CONTENT,
    )
    .await;
    let deadline = Instant::now() + WAIT;
    while !containers_with(&docker, &format!("brume.service={service_id}"))
        .await
        .is_empty()
    {
        assert!(Instant::now() < deadline, "the service's containers stay");
        tokio::time::sleep(POLL).await;
    }
    assert!(api.service(&service_id).await.is_none());
    eprintln!("step 9: service deleted, no container left");

    // 10. The bucket: no secret, and the segments glued together rebuild the hash chain.
    let events = api
        .get(&format!("/api/events?projectId={project_id}&limit=200"))
        .await;
    let last_seq = events["events"]
        .as_array()
        .expect("events")
        .iter()
        .filter_map(|event| event["seq"].as_u64())
        .max()
        .expect("the project has events");
    let deadline = Instant::now() + WAIT;
    let shipped = loop {
        let shipped = check_chain(&segments(&home));
        if shipped >= last_seq {
            break shipped;
        }
        assert!(
            Instant::now() < deadline,
            "the log never reached the bucket"
        );
        tokio::time::sleep(POLL).await;
    };
    for file in segments(&home) {
        let bytes = std::fs::read(&file).expect("reads a segment");
        assert!(
            !bytes.windows(6).any(|window| window == b"s3cr3t"),
            "a secret in {}",
            file.display()
        );
    }
    eprintln!("step 10: {shipped} events in the bucket, chained, without the secret");

    public_entry(&api, &home, &created).await;
}

/// Step 11: the public entry, with a fake tunnel token.
async fn public_entry(api: &Api, home: &Path, created: &std::sync::Mutex<Created>) {
    let docker = docker().await;
    configure_ingress(home).await;
    assert_eq!(api.get("/api/ingress").await, json!({ "zone": ZONE }));

    // A service, then its domain.
    let project = api
        .call(
            Method::POST,
            "/api/projects",
            Some(json!({ "kind": "image", "image": IMAGE })),
            StatusCode::CREATED,
        )
        .await;
    let project_id = project["projectId"].as_str().expect("a project id");
    let environment_id = api
        .get("/api/environments")
        .await
        .as_array()
        .expect("an array")
        .iter()
        .find(|environment| environment["projectId"] == project_id)
        .and_then(|environment| environment["id"].as_str())
        .expect("the project's environment")
        .to_owned();
    created
        .lock()
        .expect("lock")
        .environments
        .push(environment_id.clone());
    let service_id = api
        .get("/api/services")
        .await
        .as_array()
        .expect("an array")
        .iter()
        .find(|service| service["environmentId"] == environment_id.as_str())
        .and_then(|service| service["id"].as_str())
        .expect("the image's service")
        .to_owned();
    created
        .lock()
        .expect("lock")
        .services
        .push(service_id.clone());
    let first = api.wait_new_deployment(&service_id, &[]).await;
    api.wait_status(&first, "active").await;
    // A domain needs the port that Traefik sends to.
    for (field, value) in [("port", json!(80)), ("domain", json!("demo"))] {
        api.call(
            Method::POST,
            "/api/changes",
            Some(json!({
                "serviceId": service_id,
                "type": "setting",
                "field": field,
                "value": value,
            })),
            StatusCode::CREATED,
        )
        .await;
    }
    api.call(
        Method::POST,
        &format!("/api/environments/{environment_id}/changes/deploy"),
        None,
        StatusCode::OK,
    )
    .await;
    let second = api
        .wait_new_deployment(&service_id, std::slice::from_ref(&first))
        .await;
    api.wait_status(&second, "active").await;
    eprintln!("step 11: deployment {second} active with the domain demo.{ZONE}");

    // Traefik sends the domain to nginx, and any other host to its 404.
    let host = format!("demo.{ZONE}");
    let page = wait_traefik(&docker, &host, "200").await;
    assert!(page.contains("Welcome to nginx"), "{page}");
    let (status, _) = through_traefik(&docker, &format!("other.{ZONE}")).await;
    assert_eq!(status, "404");
    eprintln!("step 11: Traefik routes {host} to nginx, other hosts to 404");

    // On the edge network, the service cannot reach Traefik: it listens on its ingress
    // address only.
    let edge = format!("brume-edge-{service_id}");
    let traefik = docker
        .inspect_container("brume-traefik", None::<InspectContainerOptions>)
        .await
        .expect("Traefik runs");
    let edge_address = traefik
        .network_settings
        .and_then(|settings| settings.networks)
        .and_then(|networks| networks.get(&edge).cloned())
        .and_then(|endpoint| endpoint.ip_address)
        .expect("Traefik is on the edge network");
    let url = format!("http://{edge_address}:8080/");
    let (code, _) = curl(
        &docker,
        &edge,
        &[
            "-s",
            "--max-time",
            "5",
            "-H",
            &format!("Host: {host}"),
            &url,
        ],
    )
    .await;
    // 7: the connection is refused.
    assert_eq!(code, 7, "Traefik answered on {url}");
    eprintln!("step 11: Traefik refuses connections on its edge address");

    // cloudflared runs with the fake token, and fails without stopping the rest: on the token,
    // not on its arguments.
    let cloudflared = containers_with(&docker, "brume.component=cloudflared").await;
    assert_eq!(cloudflared.len(), 1, "one cloudflared");
    let options = LogsOptionsBuilder::new().stdout(true).stderr(true).build();
    let mut logs = docker.logs(&cloudflared[0], Some(options));
    let mut output = String::new();
    while let Some(chunk) = logs.next().await {
        output.push_str(&chunk.expect("a log chunk").to_string());
    }
    eprintln!("step 11: cloudflared says: {}", output.trim());
    assert!(
        !output.contains("flag provided but not defined"),
        "{output}"
    );
    assert!(!output.contains("s3cr3t"), "cloudflared logged its token");
    let config_file = home.join("agent/traefik/dynamic.yml");
    let mode = std::fs::metadata(&config_file)
        .expect("the config of Traefik")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o644);
    let config = std::fs::read_to_string(&config_file).expect("reads the config");
    assert!(config.contains(&format!("Host(`{host}`)")), "{config}");
    assert!(!config.contains("s3cr3t"), "a secret in Traefik's config");
    eprintln!("step 11: cloudflared fails with the fake token, Traefik's config is 0644");

    // The service goes, and its route with it.
    api.call(
        Method::DELETE,
        &format!("/api/services/{service_id}"),
        None,
        StatusCode::NO_CONTENT,
    )
    .await;
    wait_traefik(&docker, &host, "404").await;
    eprintln!("step 11: service deleted, {host} gets a 404");
}

#[tokio::test]
#[ignore = "runs the brume binary: cargo test -p brume --test e2e -- --ignored"]
async fn dev_survives_docker_unreachable() {
    let dir = tempfile::Builder::new()
        .prefix("brume-e2e")
        .tempdir_in("/tmp")
        .expect("a temporary directory");
    // An earlier home, which the confirmed reset deletes.
    std::fs::create_dir_all(dir.path().join("home")).expect("creates the home");
    std::fs::write(dir.path().join("home/stale"), b"old").expect("writes");
    let nowhere = PathBuf::from("unix:///nonexistent");
    // `HOME` too: the agent falls back to `~/.docker/run/docker.sock`.
    let fake_home = dir.path().to_path_buf();
    let mut dev = Dev::start_in(
        dir,
        &[("DOCKER_HOST", &nowhere), ("HOME", &fake_home)],
        &["--reset"],
        Some("y\n"),
    );
    assert!(!dev.home().join("stale").exists(), "the reset deleted it");

    let api = Api::login(&dev).await;
    let projects = api.get("/api/projects").await;
    assert_eq!(projects, json!([]));
    let project = api
        .call(
            Method::POST,
            "/api/projects",
            Some(json!({ "kind": "image", "image": IMAGE })),
            StatusCode::CREATED,
        )
        .await;
    assert!(project["projectId"].is_string());

    // Without Docker, the image never resolves: the deployment stays `building`.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut statuses = BTreeMap::new();
    while Instant::now() < deadline {
        for deployment in api
            .get("/api/deployments")
            .await
            .as_array()
            .expect("an array")
        {
            statuses.insert(
                deployment["id"].as_str().unwrap_or_default().to_owned(),
                deployment["status"].as_str().unwrap_or_default().to_owned(),
            );
        }
        tokio::time::sleep(POLL).await;
    }
    assert_eq!(statuses.len(), 1, "{statuses:?}");
    assert!(
        statuses.values().all(|status| status == "building"),
        "{statuses:?}"
    );
    assert_eq!(
        api.get("/api/projects").await.as_array().map(Vec::len),
        Some(1)
    );

    let status = dev.interrupt();
    assert!(
        status.success(),
        "brume dev exited with {status}:\n{}",
        dev.log()
    );
    assert!(dev.log().contains("Docker is unreachable"), "{}", dev.log());
}

#[test]
#[ignore = "runs the brume binary: cargo test -p brume --test e2e -- --ignored"]
fn dev_stops_everything_when_a_service_dies() {
    let dir = tempfile::Builder::new()
        .prefix("brume-e2e")
        .tempdir_in("/tmp")
        .expect("a temporary directory");
    // No Docker: this agent must not touch the containers of this machine.
    let nowhere = PathBuf::from("unix:///nonexistent");
    let fake_home = dir.path().to_path_buf();
    let mut dev = Dev::start_in(
        dir,
        &[("DOCKER_HOST", &nowhere), ("HOME", &fake_home)],
        &[],
        None,
    );
    // telemetry, engine, agent, api: the agent dies.
    let agent = dev.pids[2];
    assert!(signal("-KILL", agent));
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = dev.child.try_wait().expect("waits") {
            break status;
        }
        assert!(Instant::now() < deadline, "brume dev still runs");
        std::thread::sleep(POLL);
    };
    let log = dev.log();
    assert_eq!(status.code(), Some(1), "{log}");
    assert!(
        log.contains("The agent service stopped (signal: 9"),
        "{log}"
    );
    for pid in dev.pids.clone() {
        assert!(!signal("-0", pid), "service {pid} still runs");
    }
}
