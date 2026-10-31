//! The HTTP probe of a starting container: a throwaway `busybox` container on the same network
//! runs `wget`, and only its exit code counts. It has no egress: it joins only the network of
//! the probed container.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use bollard::Docker;
use bollard::models::{
    ContainerCreateBody, HostConfig, HostConfigLogConfig, RestartPolicy, RestartPolicyNameEnum,
};
use bollard::query_parameters::{
    CreateContainerOptions, RemoveContainerOptionsBuilder, StartContainerOptions,
    WaitContainerOptions,
};
use futures_util::StreamExt;
use tokio::time::timeout;

use crate::jobs::pull;

/// busybox 1.37, pinned by the digest of its multi-arch index.
pub const PROBE_IMAGE: &str =
    "busybox@sha256:a7dbc319b1438115a207e11930dc4bb3e643dcb3ec0bdd694bb36cd8fad289ec";

/// Marks the probe containers, which Brume removes after use. They are not `brume.managed`:
/// `plan` would remove them as unwanted.
pub const LABEL_PROBE: &str = "brume.probe";

/// How long `wget` waits for the server.
const WGET_TIMEOUT_SECS: u32 = 3;
/// How long the probe container may take, from start to exit.
const PROBE_DEADLINE: Duration = Duration::from_secs(15);
const PROBE_MEMORY_BYTES: i64 = 32 * 1024 * 1024;
/// A tenth of a CPU.
const PROBE_NANO_CPUS: i64 = 100_000_000;
const PROBE_PIDS: i64 = 32;

/// Whether `GET http://<container>:<port><path>` succeeds from `network`. Every error is a
/// failure. The probe container carries `extra_labels`, and is removed in every case.
pub async fn probe(
    docker: &Docker,
    extra_labels: &BTreeMap<String, String>,
    network: &str,
    container: &str,
    port: u16,
    path: &str,
) -> bool {
    if docker.inspect_image(PROBE_IMAGE).await.is_err()
        && let Err(error) = pull(docker, PROBE_IMAGE).await
    {
        tracing::warn!(%error, "cannot pull the probe image");
        return false;
    }
    let mut labels: HashMap<String, String> = extra_labels.clone().into_iter().collect();
    labels.insert(LABEL_PROBE.to_owned(), "1".to_owned());
    let body = ContainerCreateBody {
        image: Some(PROBE_IMAGE.to_owned()),
        cmd: Some(wget_command(container, port, path)),
        labels: Some(labels),
        host_config: Some(HostConfig {
            network_mode: Some(network.to_owned()),
            cap_drop: Some(vec!["ALL".to_owned()]),
            security_opt: Some(vec!["no-new-privileges:true".to_owned()]),
            memory: Some(PROBE_MEMORY_BYTES),
            memory_swap: Some(PROBE_MEMORY_BYTES),
            nano_cpus: Some(PROBE_NANO_CPUS),
            pids_limit: Some(PROBE_PIDS),
            restart_policy: Some(RestartPolicy {
                name: Some(RestartPolicyNameEnum::NO),
                maximum_retry_count: None,
            }),
            log_config: Some(HostConfigLogConfig {
                typ: Some("none".to_owned()),
                config: None,
            }),
            privileged: Some(false),
            publish_all_ports: Some(false),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    };
    let id = match docker
        .create_container(None::<CreateContainerOptions>, body)
        .await
    {
        Ok(created) => created.id,
        Err(error) => {
            tracing::warn!(%error, container, "cannot create the probe container");
            return false;
        }
    };
    let success = timeout(PROBE_DEADLINE, run_to_exit(docker, &id))
        .await
        .unwrap_or(false);
    let options = RemoveContainerOptionsBuilder::new().force(true).build();
    if let Err(error) = docker.remove_container(&id, Some(options)).await {
        tracing::warn!(%error, "cannot remove a probe container");
    }
    success
}

/// Starts the probe container and waits for its exit: success is exit code 0.
async fn run_to_exit(docker: &Docker, id: &str) -> bool {
    if docker
        .start_container(id, None::<StartContainerOptions>)
        .await
        .is_err()
    {
        return false;
    }
    let mut exit = docker.wait_container(id, None::<WaitContainerOptions>);
    matches!(exit.next().await, Some(Ok(response)) if response.status_code == 0)
}

/// The argv of the probe.
pub(crate) fn wget_command(container: &str, port: u16, path: &str) -> Vec<String> {
    let slash = if path.starts_with('/') { "" } else { "/" };
    let timeout = WGET_TIMEOUT_SECS.to_string();
    let url = format!("http://{container}:{port}{slash}{path}");
    ["wget", "-q", "-O", "/dev/null", "-T", &timeout, &url]
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::wget_command;

    #[test]
    fn wget_gets_the_path_on_the_container() {
        assert_eq!(
            wget_command("brume-web", 8080, "/health"),
            [
                "wget",
                "-q",
                "-O",
                "/dev/null",
                "-T",
                "3",
                "http://brume-web:8080/health"
            ]
        );
        assert_eq!(
            wget_command("brume-web", 80, "health")[6],
            "http://brume-web:80/health"
        );
    }
}
