//! `plan` against an in-memory Docker: from any snapshot, applying its actions until it has none
//! left reaches the desired state without touching an unmanaged container. Then the backoff of
//! restarts and the health of containers.

use std::collections::BTreeSet;

use brume_agent_core::backoff::Restarts;
use brume_agent_core::config::{
    LABEL_CONFIG_HASH, LABEL_DEPLOYMENT, LABEL_MANAGED, container_config,
};
use brume_agent_core::health::{ProbeResults, report};
use brume_agent_core::observed::{DockerSnapshot, SnapContainer, SnapNetwork, SnapState};
use brume_agent_core::plan::{Action, plan};
use brume_proto::agent::{
    ContainerHealth, ContainerSpec, DesiredState, EnvValue, EnvVar, Healthcheck, Mount,
    NetworkAttachment, NetworkSpec, VolumeSpec,
};
use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};
use proptest::sample::subsequence;

const NETWORKS: [&str; 3] = ["net-a", "net-b", "net-c"];
const VOLUMES: [&str; 2] = ["vol-a", "vol-b"];
const IMAGES: [&str; 2] = ["nginx@sha256:aa", "redis@sha256:bb"];
const ALIASES: [&str; 3] = ["api", "web", "db"];
const ROUND_MS: u64 = 2_000;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn attachment_strategy(network: &'static str) -> impl Strategy<Value = NetworkAttachment> {
    subsequence(ALIASES.to_vec(), 0..=2).prop_map(move |aliases| NetworkAttachment {
        network: network.to_owned(),
        aliases: strings(&aliases),
    })
}

fn attachments_strategy() -> impl Strategy<Value = Vec<NetworkAttachment>> {
    subsequence(NETWORKS.to_vec(), 1..=2)
        .prop_shuffle()
        .prop_flat_map(|networks| {
            networks
                .into_iter()
                .map(attachment_strategy)
                .collect::<Vec<_>>()
        })
}

fn healthcheck_strategy() -> impl Strategy<Value = Healthcheck> {
    prop_oneof![
        (0..5u32).prop_map(|grace_secs| Healthcheck::Running { grace_secs }),
        (1..60u32).prop_map(|timeout_secs| Healthcheck::Http {
            port: 80,
            path: "/".to_owned(),
            timeout_secs,
        }),
    ]
}

fn spec_strategy(index: usize) -> impl Strategy<Value = ContainerSpec> {
    (
        proptest::sample::select(IMAGES.to_vec()),
        attachments_strategy(),
        subsequence(VOLUMES.to_vec(), 0..=1),
        healthcheck_strategy(),
        any::<bool>(),
        1..4u32,
    )
        .prop_map(
            move |(image, networks, volumes, healthcheck, secret, deployment)| ContainerSpec {
                name: format!("brume-c{index}"),
                image: image.to_owned(),
                environment_id: "env".to_owned(),
                service_id: format!("svc-{index}"),
                deployment_id: format!("dep-{deployment}"),
                env: vec![EnvVar {
                    name: "TOKEN".to_owned(),
                    value: if secret {
                        EnvValue::Secret("s3cr3t".into())
                    } else {
                        EnvValue::Plain("plain".to_owned())
                    },
                }],
                command: None,
                networks,
                mounts: volumes
                    .into_iter()
                    .map(|volume| Mount {
                        volume: volume.to_owned(),
                        path: format!("/data/{volume}"),
                    })
                    .collect(),
                cpu_millis: 500,
                memory_mb: 128,
                stop_timeout_secs: 10,
                healthcheck,
            },
        )
}

fn desired_strategy() -> impl Strategy<Value = DesiredState> {
    (0..4usize)
        .prop_flat_map(|count| (0..count).map(spec_strategy).collect::<Vec<_>>())
        .prop_map(|containers| {
            let networks = NETWORKS
                .iter()
                .map(|name| NetworkSpec {
                    name: (*name).to_owned(),
                    internal: *name != "net-c",
                    environment_id: "env".to_owned(),
                })
                .collect();
            let volumes = VOLUMES
                .iter()
                .map(|name| VolumeSpec {
                    name: (*name).to_owned(),
                    service_id: "svc".to_owned(),
                })
                .collect();
            DesiredState {
                position: 1,
                networks,
                volumes,
                containers,
            }
        })
}

fn state_strategy() -> impl Strategy<Value = SnapState> {
    prop_oneof![
        Just(SnapState::Created),
        Just(SnapState::Running),
        (0..3i64).prop_map(|code| SnapState::Exited { code }),
        Just(SnapState::Other),
    ]
}

/// A container in the snapshot. `matches` asks for the config hash of the desired container of
/// the same name, so that it is kept rather than replaced.
#[derive(Clone, Debug)]
struct SnapSeed {
    name: String,
    managed: bool,
    matches: bool,
    state: SnapState,
    networks: Vec<NetworkAttachment>,
}

fn seed_strategy() -> impl Strategy<Value = SnapSeed> {
    let managed_name = (0..6usize).prop_map(|index| format!("brume-c{index}"));
    let other_name = (0..3usize).prop_map(|index| format!("other-{index}"));
    (
        prop_oneof![
            managed_name.prop_map(|name| (name, true)),
            other_name.prop_map(|name| (name, false)),
        ],
        any::<bool>(),
        state_strategy(),
        prop_oneof![
            attachments_strategy(),
            attachment_strategy("stray").prop_map(|attachment| vec![attachment]),
        ],
    )
        .prop_map(|((name, managed), matches, state, networks)| SnapSeed {
            name,
            managed,
            matches,
            state,
            networks,
        })
}

fn snapshot_strategy() -> impl Strategy<
    Value = (
        Vec<&'static str>,
        Vec<&'static str>,
        Vec<&'static str>,
        Vec<SnapSeed>,
    ),
> {
    (
        subsequence(NETWORKS.to_vec(), 0..=3),
        subsequence(VOLUMES.to_vec(), 0..=2),
        subsequence(IMAGES.to_vec(), 0..=2),
        vec(seed_strategy(), 0..6),
    )
}

fn build_snapshot(
    desired: &DesiredState,
    (networks, volumes, images, seeds): (Vec<&str>, Vec<&str>, Vec<&str>, Vec<SnapSeed>),
) -> DockerSnapshot {
    let mut names = BTreeSet::new();
    let containers = seeds
        .into_iter()
        .filter(|seed| names.insert(seed.name.clone()))
        .map(|seed| {
            let wanted = desired
                .containers
                .iter()
                .find(|spec| spec.name == seed.name);
            let config_hash = match wanted {
                Some(spec) if seed.matches => container_config(spec).config_hash().to_owned(),
                _ => "stale".to_owned(),
            };
            SnapContainer {
                name: seed.name,
                managed: seed.managed,
                deployment_id: "dep-0".to_owned(),
                image: IMAGES[0].to_owned(),
                state: seed.state,
                started_at_ms: (seed.state == SnapState::Running).then_some(0),
                networks: seed.networks,
                config_hash,
                stop_timeout_secs: 10,
            }
        })
        .collect();
    DockerSnapshot {
        networks: networks
            .into_iter()
            .map(|name| SnapNetwork {
                name: name.to_owned(),
                internal: name != "net-c",
            })
            .collect(),
        volumes: strings(&volumes),
        images: strings(&images),
        containers,
    }
}

/// Docker in memory. It panics on a call that the real Docker would refuse, or that touches an
/// unmanaged container.
fn apply(
    mut snapshot: DockerSnapshot,
    action: &Action,
    restarts: &mut Restarts,
    probes: &mut ProbeResults,
    now_ms: u64,
) -> DockerSnapshot {
    let managed = |snapshot: &DockerSnapshot, name: &str| -> usize {
        let index = snapshot
            .containers
            .iter()
            .position(|container| container.name == name)
            .unwrap_or_else(|| panic!("no container {name}"));
        assert!(
            snapshot.containers[index].managed,
            "touched unmanaged {name}"
        );
        index
    };
    match action {
        Action::CreateNetwork(spec) => {
            assert!(snapshot.network(&spec.name).is_none());
            snapshot.networks.push(SnapNetwork {
                name: spec.name.clone(),
                internal: spec.internal,
            });
        }
        Action::CreateVolume(spec) => {
            assert!(!snapshot.volumes.contains(&spec.name));
            snapshot.volumes.push(spec.name.clone());
        }
        Action::PullImage(image) => {
            assert!(!snapshot.images.contains(image));
            snapshot.images.push(image.clone());
        }
        Action::CreateContainer(config) => {
            assert!(snapshot.container(&config.name).is_none(), "name taken");
            assert!(snapshot.images.contains(&config.image), "image missing");
            for endpoint in &config.endpoints {
                assert!(
                    snapshot.network(&endpoint.network).is_some(),
                    "network missing"
                );
            }
            for mount in &config.host_config.mounts {
                assert!(snapshot.volumes.contains(&mount.source), "volume missing");
            }
            snapshot.containers.push(SnapContainer {
                name: config.name.clone(),
                managed: config.labels.get(LABEL_MANAGED).map(String::as_str) == Some("1"),
                deployment_id: config.labels[LABEL_DEPLOYMENT].clone(),
                image: config.image.clone(),
                state: SnapState::Created,
                started_at_ms: None,
                networks: config.endpoints.clone(),
                config_hash: config.labels[LABEL_CONFIG_HASH].clone(),
                stop_timeout_secs: config.stop_timeout_secs,
            });
        }
        Action::StartContainer(name) => {
            let index = managed(&snapshot, name);
            restarts.record_start(name, now_ms);
            let container = &mut snapshot.containers[index];
            container.state = SnapState::Running;
            container.started_at_ms = Some(now_ms);
        }
        Action::StopContainer { name, .. } => {
            let index = managed(&snapshot, name);
            snapshot.containers[index].state = SnapState::Exited { code: 0 };
        }
        Action::RemoveContainer(name) => {
            let index = managed(&snapshot, name);
            assert_ne!(
                snapshot.containers[index].state,
                SnapState::Running,
                "remove running"
            );
            snapshot.containers.remove(index);
        }
        Action::Reattach {
            container,
            network,
            aliases,
        } => {
            assert!(snapshot.network(network).is_some(), "network missing");
            let index = managed(&snapshot, container);
            let networks = &mut snapshot.containers[index].networks;
            networks.retain(|attachment| attachment.network != *network);
            networks.push(NetworkAttachment {
                network: network.clone(),
                aliases: aliases.clone(),
            });
        }
        Action::Probe { container, .. } => {
            let index = managed(&snapshot, container);
            let started_at_ms = snapshot.containers[index]
                .started_at_ms
                .expect("probed a container that never started");
            probes.record(container, started_at_ms, true, now_ms);
        }
    }
    snapshot
}

fn sorted(aliases: &[String]) -> Vec<String> {
    let set: BTreeSet<&String> = aliases.iter().collect();
    set.into_iter().cloned().collect()
}

fn attachments_match(actual: &[NetworkAttachment], wanted: &[NetworkAttachment]) -> bool {
    let normalize = |attachments: &[NetworkAttachment]| {
        let mut list: Vec<(String, Vec<String>)> = attachments
            .iter()
            .map(|attachment| (attachment.network.clone(), sorted(&attachment.aliases)))
            .collect();
        list.sort();
        list
    };
    normalize(actual) == normalize(wanted)
}

proptest! {
    #[test]
    fn plan_converges_and_never_touches_unmanaged_containers(
        desired in desired_strategy(),
        seeds in snapshot_strategy(),
    ) {
        let initial = build_snapshot(&desired, seeds);
        let mut snapshot = initial.clone();
        let mut restarts = Restarts::default();
        let mut probes = ProbeResults::default();
        let mut now_ms = 1_000_000;
        let mut converged = false;
        for _ in 0..10 {
            let actions = plan(&desired, &snapshot, &restarts, &probes, now_ms);
            if actions.is_empty() {
                converged = true;
                break;
            }
            for action in &actions {
                snapshot = apply(snapshot, action, &mut restarts, &mut probes, now_ms);
            }
            now_ms += ROUND_MS;
        }
        prop_assert!(converged, "no convergence in 10 rounds: {snapshot:#?}");

        for network in &desired.networks {
            prop_assert!(snapshot.network(&network.name).is_some());
        }
        for volume in &desired.volumes {
            prop_assert!(snapshot.volumes.contains(&volume.name));
        }
        for spec in &desired.containers {
            let container = snapshot.container(&spec.name);
            prop_assert!(container.is_some(), "missing {}", spec.name);
            let Some(container) = container else { continue };
            prop_assert!(container.managed);
            let config = container_config(spec);
            prop_assert_eq!(container.config_hash.as_str(), config.config_hash());
            prop_assert_eq!(container.state, SnapState::Running);
            prop_assert!(
                attachments_match(&container.networks, &spec.networks),
                "{:?} != {:?}", container.networks, spec.networks
            );
        }
        let unmanaged = |snapshot: &DockerSnapshot| -> Vec<SnapContainer> {
            snapshot.containers.iter().filter(|container| !container.managed).cloned().collect()
        };
        prop_assert_eq!(unmanaged(&snapshot), unmanaged(&initial));
        for container in snapshot.containers.iter().filter(|container| container.managed) {
            prop_assert!(desired.containers.iter().any(|spec| spec.name == container.name));
        }
    }
}

fn nginx(healthcheck: Healthcheck) -> ContainerSpec {
    ContainerSpec {
        name: "brume-web".to_owned(),
        image: IMAGES[0].to_owned(),
        environment_id: "env".to_owned(),
        service_id: "svc".to_owned(),
        deployment_id: "dep".to_owned(),
        env: Vec::new(),
        command: None,
        networks: vec![NetworkAttachment {
            network: "net-a".to_owned(),
            aliases: strings(&["web"]),
        }],
        mounts: Vec::new(),
        cpu_millis: 500,
        memory_mb: 128,
        stop_timeout_secs: 10,
        healthcheck,
    }
}

fn one(spec: &ContainerSpec) -> DesiredState {
    DesiredState {
        position: 1,
        networks: vec![NetworkSpec {
            name: "net-a".to_owned(),
            internal: true,
            environment_id: "env".to_owned(),
        }],
        volumes: Vec::new(),
        containers: vec![spec.clone()],
    }
}

fn running(spec: &ContainerSpec, state: SnapState, started_at_ms: u64) -> DockerSnapshot {
    DockerSnapshot {
        networks: vec![SnapNetwork {
            name: "net-a".to_owned(),
            internal: true,
        }],
        volumes: Vec::new(),
        images: strings(&[IMAGES[0]]),
        containers: vec![SnapContainer {
            name: spec.name.clone(),
            managed: true,
            deployment_id: spec.deployment_id.clone(),
            image: spec.image.clone(),
            state,
            started_at_ms: Some(started_at_ms),
            networks: spec.networks.clone(),
            config_hash: container_config(spec).config_hash().to_owned(),
            stop_timeout_secs: 10,
        }],
    }
}

fn health(observed: &brume_proto::agent::ObservedState) -> ContainerHealth {
    observed.containers[0].health.clone()
}

#[test]
fn crashed_container_restarts_with_backoff() {
    let spec = nginx(Healthcheck::Running { grace_secs: 5 });
    let desired = one(&spec);
    let crashed = running(&spec, SnapState::Exited { code: 1 }, 0);
    let probes = ProbeResults::default();
    let start = vec![Action::StartContainer(spec.name.clone())];

    let mut restarts = Restarts::default();
    restarts.record_start(&spec.name, 10_000);
    // Waits of 1 s, 2 s, 4 s… after each start, up to 60 s.
    let mut now_ms = 10_000;
    for wait_ms in [1_000, 2_000, 4_000, 8_000, 16_000, 32_000, 60_000, 60_000] {
        assert!(plan(&desired, &crashed, &restarts, &probes, now_ms + wait_ms - 1).is_empty());
        assert_eq!(
            plan(&desired, &crashed, &restarts, &probes, now_ms + wait_ms),
            start
        );
        now_ms += wait_ms;
        restarts.record_start(&spec.name, now_ms);
    }
    assert_eq!(restarts.restarts(&spec.name), 8);
    let observed = report(&desired, &crashed, &probes, &restarts, now_ms);
    assert_eq!(health(&observed), ContainerHealth::Exited { code: 1 });
    assert_eq!(observed.containers[0].restarts, 8);

    // A reset brings the wait back to 1 s and keeps the count.
    restarts.reset(&spec.name);
    restarts.record_start(&spec.name, now_ms);
    assert!(plan(&desired, &crashed, &restarts, &probes, now_ms + 999).is_empty());
    assert_eq!(
        plan(&desired, &crashed, &restarts, &probes, now_ms + 1_000),
        start
    );
    assert_eq!(restarts.restarts(&spec.name), 9);

    // Running for 60 s resets the wait; a container gone is forgotten.
    let stable = running(&spec, SnapState::Running, now_ms);
    for _ in 0..5 {
        restarts.record_start(&spec.name, now_ms);
    }
    restarts.settle(&stable, now_ms + 60_000);
    assert!(restarts.allows_start(&spec.name, now_ms + 60_000));
    restarts.settle(&DockerSnapshot::default(), now_ms);
    assert_eq!(restarts.restarts(&spec.name), 0);
}

#[test]
fn running_grace_then_healthy() {
    let spec = nginx(Healthcheck::Running { grace_secs: 5 });
    let desired = one(&spec);
    let snapshot = running(&spec, SnapState::Running, 100_000);
    let probes = ProbeResults::default();
    let mut restarts = Restarts::default();
    restarts.record_start(&spec.name, 100_000);

    let at = |now_ms| report(&desired, &snapshot, &probes, &restarts, now_ms);
    assert_eq!(health(&at(104_999)), ContainerHealth::Starting);
    assert_eq!(health(&at(105_000)), ContainerHealth::Healthy);
    let observed = at(105_000);
    assert_eq!(observed.containers[0].name, "brume-web");
    assert_eq!(observed.containers[0].deployment_id, "dep");
    assert_eq!(observed.containers[0].aliases, strings(&["web"]));
    assert_eq!(observed.containers[0].restarts, 0);

    // No probe for a `Running` healthcheck, and nothing else to do.
    assert!(plan(&desired, &snapshot, &restarts, &probes, 104_000).is_empty());

    let missing = report(&desired, &DockerSnapshot::default(), &probes, &restarts, 0);
    assert_eq!(health(&missing), ContainerHealth::Missing);
}

#[test]
fn http_probe_timeout_is_unhealthy() {
    let spec = nginx(Healthcheck::Http {
        port: 8080,
        path: "/health".to_owned(),
        timeout_secs: 10,
    });
    let desired = one(&spec);
    let snapshot = running(&spec, SnapState::Running, 100_000);
    let restarts = Restarts::default();
    let mut probes = ProbeResults::default();
    let probe = vec![Action::Probe {
        container: spec.name.clone(),
        network: "net-a".to_owned(),
        port: 8080,
        path: "/health".to_owned(),
    }];

    assert_eq!(
        plan(&desired, &snapshot, &restarts, &probes, 100_000),
        probe
    );
    probes.record(&spec.name, 100_000, false, 100_000);
    // Once every 2 s.
    assert!(plan(&desired, &snapshot, &restarts, &probes, 101_999).is_empty());
    assert_eq!(
        plan(&desired, &snapshot, &restarts, &probes, 102_000),
        probe
    );
    probes.record(&spec.name, 100_000, false, 102_000);

    let at = |probes: &ProbeResults, now_ms| {
        health(&report(&desired, &snapshot, probes, &restarts, now_ms))
    };
    assert_eq!(at(&probes, 109_999), ContainerHealth::Starting);
    assert!(matches!(
        at(&probes, 110_000),
        ContainerHealth::Unhealthy { .. }
    ));
    // An unhealthy container is not probed any more.
    assert!(plan(&desired, &snapshot, &restarts, &probes, 110_000).is_empty());

    // A success before the timeout makes it healthy, and stops the probes.
    let mut healthy = ProbeResults::default();
    healthy.record(&spec.name, 100_000, true, 104_000);
    assert_eq!(at(&healthy, 104_000), ContainerHealth::Healthy);
    assert_eq!(at(&healthy, 200_000), ContainerHealth::Healthy);
    assert!(plan(&desired, &snapshot, &restarts, &healthy, 200_000).is_empty());

    // A success of an earlier start does not count after a restart.
    let restarted = running(&spec, SnapState::Running, 300_000);
    let observed = report(&desired, &restarted, &healthy, &restarts, 300_000);
    assert_eq!(health(&observed), ContainerHealth::Starting);
    assert_eq!(
        plan(&desired, &restarted, &restarts, &healthy, 300_000),
        probe
    );
}

#[test]
fn unmanaged_container_with_a_wanted_name_is_left_alone() {
    let spec = nginx(Healthcheck::Running { grace_secs: 0 });
    let desired = one(&spec);
    let mut snapshot = running(&spec, SnapState::Exited { code: 0 }, 0);
    snapshot.containers[0].managed = false;
    snapshot.containers[0].config_hash = "stale".to_owned();
    let actions = plan(
        &desired,
        &snapshot,
        &Restarts::default(),
        &ProbeResults::default(),
        0,
    );
    assert!(actions.is_empty(), "{actions:?}");
}

#[test]
fn network_with_the_wrong_isolation_blocks_the_container() {
    let spec = nginx(Healthcheck::Running { grace_secs: 0 });
    let desired = one(&spec);
    let snapshot = DockerSnapshot {
        networks: vec![SnapNetwork {
            name: "net-a".to_owned(),
            internal: false,
        }],
        volumes: Vec::new(),
        images: strings(&[IMAGES[0]]),
        containers: Vec::new(),
    };
    let actions = plan(
        &desired,
        &snapshot,
        &Restarts::default(),
        &ProbeResults::default(),
        0,
    );
    assert!(
        !actions
            .iter()
            .any(|action| matches!(action, Action::CreateContainer(_))),
        "{actions:?}"
    );
}

#[test]
fn actions_follow_the_documented_order() {
    let wanted = nginx(Healthcheck::Running { grace_secs: 0 });
    let mut desired = one(&wanted);
    desired.volumes.push(VolumeSpec {
        name: "vol-a".to_owned(),
        service_id: "svc".to_owned(),
    });
    let mut crashed = nginx(Healthcheck::Running { grace_secs: 0 });
    crashed.name = "brume-crashed".to_owned();
    let mut moved = nginx(Healthcheck::Running { grace_secs: 0 });
    moved.name = "brume-moved".to_owned();
    desired.containers.extend([crashed.clone(), moved.clone()]);

    let mut snapshot = running(&crashed, SnapState::Exited { code: 2 }, 0);
    let mut moved_container = running(&moved, SnapState::Running, 0).containers.remove(0);
    moved_container.networks[0].aliases = strings(&["old"]);
    let mut old = running(&wanted, SnapState::Running, 0).containers.remove(0);
    old.name = "brume-old".to_owned();
    snapshot.containers.extend([moved_container, old]);
    snapshot.networks.clear();
    snapshot.images.clear();

    let actions = plan(
        &desired,
        &snapshot,
        &Restarts::default(),
        &ProbeResults::default(),
        0,
    );
    let kinds: Vec<&str> = actions
        .iter()
        .map(|action| match action {
            Action::CreateNetwork(_) => "network",
            Action::CreateVolume(_) => "volume",
            Action::PullImage(_) => "pull",
            Action::CreateContainer(_) => "create",
            Action::StartContainer(_) => "start",
            Action::StopContainer { .. } => "stop",
            Action::RemoveContainer(_) => "remove",
            Action::Reattach { .. } => "reattach",
            Action::Probe { .. } => "probe",
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "network", "volume", "pull", "create", "start", "start", "reattach", "stop", "remove"
        ]
    );
    assert_eq!(actions[4], Action::StartContainer("brume-web".to_owned()));
    assert_eq!(
        actions[5],
        Action::StartContainer("brume-crashed".to_owned())
    );
    assert_eq!(
        actions[7],
        Action::StopContainer {
            name: "brume-old".to_owned(),
            timeout_secs: 10
        }
    );
}
