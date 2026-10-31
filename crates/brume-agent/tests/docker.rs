//! Against the real Docker: `cargo test -p brume-agent -- --ignored`. Everything the test
//! creates is labelled `brume.test=1` and removed at the end, even after a failure.

use std::collections::{BTreeSet, HashMap};
use std::panic::AssertUnwindSafe;
use std::time::Duration;

use bollard::Docker;
use bollard::errors::Error;
use bollard::query_parameters::{
    InspectContainerOptions, ListContainersOptionsBuilder, ListNetworksOptionsBuilder,
    ListVolumesOptionsBuilder, RemoveContainerOptionsBuilder, RemoveVolumeOptions,
};
use brume_agent::Reconciler;
use brume_agent::docker::{DockerShell, connect};
use brume_agent::jobs::resolve_image;
use brume_agent::logs::LogForwarder;
use brume_proto::agent::{
    ContainerHealth, ContainerSpec, DesiredState, EnvValue, EnvVar, Healthcheck, JobOutcome, Mount,
    NetworkAttachment, NetworkSpec, VolumeSpec,
};
use brume_proto::telemetry::LogBatch;
use futures_util::FutureExt;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;

const TEST_LABEL: &str = "brume.test";

fn test_filter() -> HashMap<&'static str, Vec<&'static str>> {
    HashMap::from([("label", vec!["brume.test=1"])])
}

/// Removes every container, network and volume labelled `brume.test=1`. A failure is printed,
/// not fatal: the test's own outcome matters more.
async fn cleanup(docker: &Docker) {
    let options = ListContainersOptionsBuilder::new()
        .all(true)
        .filters(&test_filter())
        .build();
    match docker.list_containers(Some(options)).await {
        Ok(containers) => {
            for id in containers.into_iter().filter_map(|container| container.id) {
                let options = RemoveContainerOptionsBuilder::new()
                    .force(true)
                    .v(true)
                    .build();
                if let Err(error) = docker.remove_container(&id, Some(options)).await {
                    eprintln!("cleanup: cannot remove container {id}: {error}");
                }
            }
        }
        Err(error) => eprintln!("cleanup: cannot list containers: {error}"),
    }
    let options = ListNetworksOptionsBuilder::new()
        .filters(&test_filter())
        .build();
    match docker.list_networks(Some(options)).await {
        Ok(networks) => {
            for name in networks.into_iter().filter_map(|network| network.name) {
                if let Err(error) = docker.remove_network(&name).await {
                    eprintln!("cleanup: cannot remove network {name}: {error}");
                }
            }
        }
        Err(error) => eprintln!("cleanup: cannot list networks: {error}"),
    }
    let options = ListVolumesOptionsBuilder::new()
        .filters(&test_filter())
        .build();
    match docker.list_volumes(Some(options)).await {
        Ok(list) => {
            for volume in list.volumes.unwrap_or_default() {
                let removed = docker
                    .remove_volume(&volume.name, None::<RemoveVolumeOptions>)
                    .await;
                if let Err(error) = removed {
                    eprintln!("cleanup: cannot remove volume {}: {error}", volume.name);
                }
            }
        }
        Err(error) => eprintln!("cleanup: cannot list volumes: {error}"),
    }
}

fn desired(suffix: &str, image: &str) -> DesiredState {
    let network = format!("brume-test-env-{suffix}");
    let volume = format!("brume-vol-test-{suffix}");
    DesiredState {
        position: 1,
        networks: vec![NetworkSpec {
            name: network.clone(),
            internal: true,
            environment_id: "env-test".to_owned(),
        }],
        volumes: vec![VolumeSpec {
            name: volume.clone(),
            service_id: "svc-test".to_owned(),
        }],
        containers: vec![ContainerSpec {
            name: format!("brume-test-web-{suffix}"),
            image: image.to_owned(),
            environment_id: "env-test".to_owned(),
            service_id: "svc-test".to_owned(),
            deployment_id: "dep-test".to_owned(),
            env: vec![
                EnvVar {
                    name: "GREETING".to_owned(),
                    value: EnvValue::Plain("hello".to_owned()),
                },
                EnvVar {
                    name: "TOKEN".to_owned(),
                    value: EnvValue::Secret("s3cr3t".into()),
                },
            ],
            command: None,
            networks: vec![NetworkAttachment {
                network,
                aliases: vec!["web".to_owned()],
            }],
            mounts: vec![Mount {
                volume,
                path: "/var/cache/nginx".to_owned(),
            }],
            cpu_millis: 250,
            memory_mb: 64,
            stop_timeout_secs: 5,
            healthcheck: Healthcheck::Http {
                port: 80,
                path: "/".to_owned(),
                timeout_secs: 60,
            },
        }],
    }
}

async fn scenario(docker: &Docker) {
    let outcome = resolve_image(docker, "nginx:1.29-alpine").await;
    let JobOutcome::ImageResolved {
        digest,
        exposed_ports,
        ..
    } = outcome
    else {
        panic!("not resolved: {outcome:?}");
    };
    assert!(digest.starts_with("nginx@sha256:"), "{digest}");
    assert!(exposed_ports.contains(&80), "{exposed_ports:?}");

    let suffix = format!("{}", std::process::id());
    let mut state = desired(&suffix, &digest);
    let name = state.containers[0].name.clone();
    let mut reconciler = Reconciler::new(DockerShell::new(docker.clone()).scoped(TEST_LABEL, "1"));

    let mut health = ContainerHealth::Missing;
    for _ in 0..60 {
        let round = reconciler.reconcile(&state).await.expect("reconciles");
        health = round.observed.containers[0].health.clone();
        if health == ContainerHealth::Healthy {
            assert_eq!(round.observed.containers[0].aliases, ["web"]);
            assert!(
                round
                    .sources
                    .get(&name)
                    .is_some_and(|source| source.running)
            );
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert_eq!(health, ContainerHealth::Healthy);
    forwards_logs(&mut reconciler, &state).await;

    let inspect = docker
        .inspect_container(&name, None::<InspectContainerOptions>)
        .await
        .expect("inspects");
    let host_config = inspect.host_config.expect("host config");
    assert_eq!(host_config.cap_drop, Some(vec!["ALL".to_owned()]));
    let cap_add: BTreeSet<String> = host_config
        .cap_add
        .unwrap_or_default()
        .into_iter()
        .collect();
    let expected: BTreeSet<String> = [
        "CHOWN",
        "DAC_OVERRIDE",
        "FOWNER",
        "FSETID",
        "KILL",
        "SETGID",
        "SETUID",
        "NET_BIND_SERVICE",
    ]
    .iter()
    .map(|cap| (*cap).to_owned())
    .collect();
    assert_eq!(cap_add, expected);
    assert_eq!(
        host_config.security_opt,
        Some(vec!["no-new-privileges:true".to_owned()])
    );
    assert_eq!(host_config.privileged, Some(false));
    assert!(host_config.port_bindings.unwrap_or_default().is_empty());
    assert_eq!(host_config.memory, Some(64 * 1024 * 1024));
    assert_eq!(host_config.pids_limit, Some(512));
    let config = inspect.config.expect("config");
    assert!(
        config
            .env
            .unwrap_or_default()
            .contains(&"TOKEN=s3cr3t".to_owned())
    );
    let labels = config.labels.unwrap_or_default();
    assert_eq!(labels.get("brume.managed").map(String::as_str), Some("1"));
    assert_eq!(labels.get(TEST_LABEL).map(String::as_str), Some("1"));

    reattaches_with_new_aliases(docker, &mut reconciler, &mut state).await;

    // Out of the desired state, the container goes.
    state.position = 3;
    state.containers.clear();
    let mut gone = false;
    for _ in 0..30 {
        reconciler.reconcile(&state).await.expect("reconciles");
        match docker
            .inspect_container(&name, None::<InspectContainerOptions>)
            .await
        {
            Err(Error::DockerResponseServerError {
                status_code: 404, ..
            }) => {
                gone = true;
                break;
            }
            _ => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
    assert!(gone, "{name} is still there");
}

/// New aliases in the desired state reach the running container through `Reattach`: the same
/// container, never restarted, answers to the new names only.
async fn reattaches_with_new_aliases(
    docker: &Docker,
    reconciler: &mut Reconciler,
    state: &mut DesiredState,
) {
    let name = state.containers[0].name.clone();
    let network = state.containers[0].networks[0].network.clone();
    let before = docker
        .inspect_container(&name, None::<InspectContainerOptions>)
        .await
        .expect("inspects");
    let started_at = before.state.and_then(|state| state.started_at);

    state.position = 2;
    state.containers[0].networks[0].aliases = vec!["www".to_owned(), "web-2".to_owned()];
    let mut observed = None;
    for _ in 0..30 {
        let round = reconciler.reconcile(state).await.expect("reconciles");
        let container = round.observed.containers[0].clone();
        if container.aliases == ["web-2", "www"] {
            observed = Some(container);
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let observed = observed.expect("the aliases change");
    assert_eq!(observed.health, ContainerHealth::Healthy);

    let after = docker
        .inspect_container(&name, None::<InspectContainerOptions>)
        .await
        .expect("inspects");
    assert_eq!(after.id, before.id, "{name} was recreated");
    let state_after = after.state.expect("state");
    assert_eq!(state_after.running, Some(true));
    assert_eq!(state_after.started_at, started_at, "{name} was restarted");
    let aliases: BTreeSet<String> = after
        .network_settings
        .and_then(|settings| settings.networks)
        .and_then(|mut networks| networks.remove(&network))
        .and_then(|endpoint| endpoint.aliases)
        .unwrap_or_default()
        .into_iter()
        .collect();
    assert!(aliases.contains("www"), "{aliases:?}");
    assert!(aliases.contains("web-2"), "{aliases:?}");
    assert!(!aliases.contains("web"), "{aliases:?}");
}

/// nginx's entrypoint writes to stdout and stderr at start: those lines reach a fake telemetry.
async fn forwards_logs(reconciler: &mut Reconciler, state: &DesiredState) {
    let dir = tempfile::tempdir().expect("temp dir");
    let socket = dir.path().join("ingest.sock");
    let listener = UnixListener::bind(&socket).expect("binds");
    let mut logs = LogForwarder::start(&socket);
    let round = reconciler.reconcile(state).await.expect("reconciles");
    logs.update(reconciler.shell().docker(), &round.sources)
        .await;

    let (mut stream, _) = tokio::time::timeout(Duration::from_secs(10), listener.accept())
        .await
        .expect("connects")
        .expect("accepts");
    let mut header = [0; 4];
    stream.read_exact(&mut header).await.expect("reads");
    let mut body = vec![0; usize::try_from(u32::from_be_bytes(header)).expect("fits")];
    stream.read_exact(&mut body).await.expect("reads");
    let batch: LogBatch = serde_json::from_slice(&body).expect("decodes");
    assert!(!batch.lines.is_empty());
    let line = &batch.lines[0];
    assert_eq!(line.environment_id, "env-test");
    assert_eq!(line.service_id, "svc-test");
    assert_eq!(line.deployment_id, "dep-test");
    assert!(
        line.timestamp_ms > 1_700_000_000_000,
        "{}",
        line.timestamp_ms
    );
    assert!(!line.message.is_empty());
    assert!(!line.message.ends_with('\n'));
}

#[tokio::test]
#[ignore = "needs Docker"]
async fn reconciles_nginx_and_resolves_image() {
    let docker = connect().await.expect("reaches Docker");
    let result = AssertUnwindSafe(scenario(&docker)).catch_unwind().await;
    cleanup(&docker).await;
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
