//! The jobs that the engine waits for. `ResolveImage` pulls an image and reports its digest, its
//! volumes and its ports.

use std::sync::Arc;
use std::time::Duration;

use bollard::Docker;
use bollard::errors::Error;
use bollard::query_parameters::CreateImageOptionsBuilder;
use brume_proto::agent::{AgentToEngine, Job, JobOutcome, MAX_FAILURE_MESSAGE_BYTES};
use futures_util::StreamExt;
use tokio::sync::{Semaphore, mpsc, watch};
use tokio::time::timeout;

/// How long a job waits for Docker before it fails.
const DOCKER_WAIT: Duration = Duration::from_secs(10);

/// The most jobs that run at once. Each one pulls an image.
pub const MAX_PARALLEL_JOBS: usize = 3;

/// Runs each job of `jobs` on its own task, `MAX_PARALLEL_JOBS` at a time, with the Docker
/// client of the moment, and sends its result to `outbox`. A job fails when Docker stays
/// unreachable for `DOCKER_WAIT` once its turn comes.
pub async fn serve(
    mut jobs: mpsc::Receiver<Job>,
    docker: watch::Receiver<Option<Docker>>,
    outbox: mpsc::Sender<AgentToEngine>,
) {
    let turns = Arc::new(Semaphore::new(MAX_PARALLEL_JOBS));
    while let Some(job) = jobs.recv().await {
        let mut docker = docker.clone();
        let outbox = outbox.clone();
        let turns = Arc::clone(&turns);
        // The task waits for its turn, not this loop: the engine link keeps handing jobs over.
        tokio::spawn(async move {
            // The semaphore is never closed.
            let Ok(_turn) = turns.acquire_owned().await else {
                return;
            };
            let client = match timeout(DOCKER_WAIT, docker.wait_for(Option::is_some)).await {
                Ok(Ok(client)) => client.clone(),
                Ok(Err(_)) | Err(_) => None,
            };
            let result = run_job(client.as_ref(), job).await;
            // A closed outbox means the agent is stopping.
            let _ = outbox.send(result).await;
        });
    }
}

/// Runs `job`, and gives its result for the engine.
async fn run_job(docker: Option<&Docker>, job: Job) -> AgentToEngine {
    match job {
        Job::ResolveImage { job_id, image } => {
            let outcome = match docker {
                Some(docker) => resolve_image(docker, &image).await,
                None => JobOutcome::Failed {
                    message: "Docker is unreachable.".to_owned(),
                },
            };
            AgentToEngine::JobResult { job_id, outcome }
        }
    }
}

/// How many times a resolve tries its pull while the registry throttles it.
const THROTTLED_ATTEMPTS: u32 = 5;

/// The wait after the first throttled pull, doubled after each next one: 2, 4, 8 then 16 s.
const THROTTLED_FIRST_WAIT: Duration = Duration::from_secs(2);

/// Pulls `image`, then reads its digest, the paths it declares as volumes and its exposed ports.
/// A pull that the registry throttles is tried again a few seconds later.
pub async fn resolve_image(docker: &Docker, image: &str) -> JobOutcome {
    let reference = pull_reference(image);
    let mut wait = THROTTLED_FIRST_WAIT;
    let mut attempt = 1;
    let message = loop {
        match pull_and_inspect(docker, &reference).await {
            Ok(outcome) => return outcome,
            Err(error) if attempt < THROTTLED_ATTEMPTS && is_throttled(&error) => {
                tracing::info!(%reference, attempt, "the registry throttles the pull, retrying");
                tokio::time::sleep(wait).await;
                wait *= 2;
                attempt += 1;
            }
            Err(Error::RequestTimeoutError) => {
                break format!(
                    "Pulling {reference} took more than {} minutes.",
                    PULL_TIMEOUT.as_secs() / 60
                );
            }
            Err(error) => break error.to_string(),
        }
    };
    JobOutcome::Failed {
        message: truncate(&message, MAX_FAILURE_MESSAGE_BYTES),
    }
}

/// Whether `error` is a registry's rate limit, which passes a few seconds later: ECR Public
/// answers anonymous pulls over about one a second with `toomanyrequests: Rate exceeded`.
fn is_throttled(error: &Error) -> bool {
    match error {
        Error::DockerResponseServerError {
            status_code: 429, ..
        } => true,
        Error::DockerResponseServerError { message, .. }
        | Error::DockerStreamError { error: message } => {
            message.to_ascii_lowercase().contains("toomanyrequests")
        }
        _ => false,
    }
}

/// Pulls `reference` to the end of Docker's progress stream, then inspects it.
pub(crate) async fn pull_and_inspect(
    docker: &Docker,
    reference: &str,
) -> Result<JobOutcome, Error> {
    pull(docker, reference).await?;
    let inspect = docker.inspect_image(reference).await?;
    let repo_digests = inspect.repo_digests.unwrap_or_default();
    let Some(digest) = pick_digest(&repo_digests, reference) else {
        return Ok(JobOutcome::Failed {
            message: format!("Docker reports no digest for {reference}."),
        });
    };
    let config = inspect.config.unwrap_or_default();
    let mut volumes = config.volumes.unwrap_or_default();
    volumes.sort();
    volumes.dedup();
    let mut exposed_ports: Vec<u16> = config
        .exposed_ports
        .unwrap_or_default()
        .iter()
        .filter_map(|key| exposed_port(key))
        .collect();
    exposed_ports.sort_unstable();
    exposed_ports.dedup();
    Ok(JobOutcome::ImageResolved {
        digest,
        volumes,
        exposed_ports,
    })
}

/// The longest a pull may take. bollard's own timeout covers the response headers only.
pub const PULL_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Pulls `reference`, following Docker's progress stream to its end, for at most
/// `PULL_TIMEOUT`.
pub(crate) async fn pull(docker: &Docker, reference: &str) -> Result<(), Error> {
    let options = CreateImageOptionsBuilder::new()
        .from_image(reference)
        .build();
    let progress = async {
        let mut progress = docker.create_image(Some(options), None, None);
        while let Some(item) = progress.next().await {
            item?;
        }
        Ok(())
    };
    timeout(PULL_TIMEOUT, progress)
        .await
        .unwrap_or(Err(Error::RequestTimeoutError))
}

/// The reference to pull: `image` with `:latest` when it has neither a tag nor a digest, since
/// Docker pulls every tag of a bare name.
pub(crate) fn pull_reference(image: &str) -> String {
    if image.contains('@') || last_component(image).contains(':') {
        image.to_owned()
    } else {
        format!("{image}:latest")
    }
}

/// The repository of a reference: without its tag nor its digest.
pub(crate) fn repository(reference: &str) -> &str {
    let without_digest = reference
        .split_once('@')
        .map_or(reference, |(name, _)| name);
    let name_start = without_digest.rfind('/').map_or(0, |slash| slash + 1);
    match without_digest[name_start..].rfind(':') {
        Some(colon) => &without_digest[..name_start + colon],
        None => without_digest,
    }
}

/// The digest reference of the image: the first of `repo_digests` in the repository of
/// `reference`, else the first one.
pub(crate) fn pick_digest(repo_digests: &[String], reference: &str) -> Option<String> {
    let prefix = format!("{}@", repository(reference));
    repo_digests
        .iter()
        .find(|digest| digest.starts_with(&prefix))
        .or_else(|| repo_digests.first())
        .cloned()
}

/// The port of an `ExposedPorts` key such as `80/tcp`.
pub(crate) fn exposed_port(key: &str) -> Option<u16> {
    key.split('/').next()?.parse().ok()
}

/// `message` cut to `max` bytes, on a character boundary.
pub(crate) fn truncate(message: &str, max: usize) -> String {
    let mut end = message.len().min(max);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    message[..end].to_owned()
}

/// The part of a reference after its last `/`.
fn last_component(reference: &str) -> &str {
    reference.rsplit('/').next().unwrap_or(reference)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bollard::errors::Error;
    use bollard::{API_DEFAULT_VERSION, Docker};
    use brume_proto::agent::Job;
    use tokio::net::UnixListener;
    use tokio::sync::{mpsc, watch};
    use tokio::time::timeout;

    use super::{
        exposed_port, is_throttled, pick_digest, pull_reference, repository, serve, truncate,
    };

    #[tokio::test]
    async fn at_most_three_pulls_run_at_once() {
        // A Docker that accepts each connection and never answers: each pull holds one.
        let dir = tempfile::tempdir().expect("temp dir");
        let socket = dir.path().join("docker.sock");
        let listener = UnixListener::bind(&socket).expect("binds");
        let docker = Docker::connect_with_unix(
            socket.to_str().expect("utf-8 path"),
            120,
            API_DEFAULT_VERSION,
        )
        .expect("client");
        let (_docker_tx, docker_rx) = watch::channel(Some(docker));
        let (jobs_tx, jobs_rx) = mpsc::channel(16);
        let (outbox_tx, _outbox_rx) = mpsc::channel(16);
        tokio::spawn(serve(jobs_rx, docker_rx, outbox_tx));
        for index in 0..5 {
            let job = Job::ResolveImage {
                job_id: format!("job-{index}"),
                image: format!("nginx:1.{index}"),
            };
            jobs_tx.send(job).await.expect("queues");
        }

        let mut held = Vec::new();
        for _ in 0..3 {
            let (stream, _) = timeout(Duration::from_secs(5), listener.accept())
                .await
                .expect("a pull starts")
                .expect("accepts");
            held.push(stream);
        }
        let fourth = timeout(Duration::from_millis(500), listener.accept()).await;
        assert!(fourth.is_err(), "a fourth pull runs");
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn bare_names_pull_latest_only() {
        assert_eq!(pull_reference("nginx"), "nginx:latest");
        assert_eq!(pull_reference("nginx:1.29-alpine"), "nginx:1.29-alpine");
        assert_eq!(
            pull_reference("ghcr.io/acme/api"),
            "ghcr.io/acme/api:latest"
        );
        assert_eq!(
            pull_reference("localhost:5000/api"),
            "localhost:5000/api:latest"
        );
        assert_eq!(
            pull_reference("localhost:5000/api:2"),
            "localhost:5000/api:2"
        );
        let pinned = format!("nginx@sha256:{}", "a".repeat(64));
        assert_eq!(pull_reference(&pinned), pinned);
    }

    #[test]
    fn digest_comes_from_the_requested_repository() {
        assert_eq!(repository("localhost:5000/api:2"), "localhost:5000/api");
        assert_eq!(repository("nginx@sha256:abc"), "nginx");
        assert_eq!(repository("nginx"), "nginx");
        let digests = strings(&["mirror.local/nginx@sha256:aa", "nginx@sha256:bb"]);
        assert_eq!(
            pick_digest(&digests, "nginx:1.29-alpine").as_deref(),
            Some("nginx@sha256:bb")
        );
        assert_eq!(
            pick_digest(&digests, "redis:8").as_deref(),
            Some("mirror.local/nginx@sha256:aa")
        );
        assert_eq!(pick_digest(&[], "nginx:latest"), None);
    }

    #[test]
    fn exposed_ports_parse_their_number() {
        assert_eq!(exposed_port("80/tcp"), Some(80));
        assert_eq!(exposed_port("53/udp"), Some(53));
        assert_eq!(exposed_port("8080"), Some(8080));
        assert_eq!(exposed_port("70000/tcp"), None);
        assert_eq!(exposed_port("http/tcp"), None);
    }

    #[test]
    fn a_registry_rate_limit_is_throttled() {
        // What Docker 29 answers when ECR Public throttles the pull.
        let ecr = Error::DockerResponseServerError {
            status_code: 429,
            message: "toomanyrequests: Rate exceeded".to_owned(),
        };
        assert!(is_throttled(&ecr));
        let other_status = Error::DockerResponseServerError {
            status_code: 500,
            message: "toomanyrequests: Rate exceeded".to_owned(),
        };
        assert!(is_throttled(&other_status));
        let mid_stream = Error::DockerStreamError {
            error: "TOOMANYREQUESTS: You have reached your pull rate limit".to_owned(),
        };
        assert!(is_throttled(&mid_stream));
        let missing = Error::DockerResponseServerError {
            status_code: 404,
            message: "manifest unknown".to_owned(),
        };
        assert!(!is_throttled(&missing));
        assert!(!is_throttled(&Error::RequestTimeoutError));
    }

    #[test]
    fn failure_messages_are_cut_to_1_kio() {
        assert_eq!(truncate("short", 1024), "short");
        let long = "é".repeat(1024);
        let cut = truncate(&long, 1024);
        assert_eq!(cut.len(), 1024);
        let odd = format!("a{long}");
        assert_eq!(truncate(&odd, 1024).len(), 1023);
    }
}
