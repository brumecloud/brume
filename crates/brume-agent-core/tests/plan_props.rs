//! `plan` against an in-memory Docker: from any snapshot, applying its actions until it has none
//! left reaches the desired state without touching an unmanaged container, network or volume.
//! Then the backoff of restarts, the health of containers, and the names and labels that keep
//! foreign networks and volumes away from containers.

use std::collections::BTreeSet;

use brume_agent_core::backoff::Restarts;
use brume_agent_core::config::{
    LABEL_CONFIG_HASH, LABEL_DEPLOYMENT, LABEL_MANAGED, container_config,
};
use brume_agent_core::health::{ProbeResults, report};
use brume_agent_core::observed::{
    DockerSnapshot, SnapContainer, SnapNetwork, SnapState, SnapVolume,
};
use brume_agent_core::plan::{Action, Refusal, RefusalReason, Resource, plan, refusals};
use brume_proto::agent::{
    ContainerHealth, ContainerSpec, DesiredState, EnvValue, EnvVar, Healthcheck, Mount,
    NetworkAttachment, NetworkSpec, VolumeSpec,
};
use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};
use proptest::sample::subsequence;

const NETWORKS: [&str; 3] = ["brume-net-a", "brume-net-b", "brume-net-c"];
const VOLUMES: [&str; 2] = ["brume-vol-a", "brume-vol-b"];
/// The one network of the desired state that has the internet.
const EGRESS: &str = "brume-net-c";
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
                    internal: *name != EGRESS,
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
                internal: name != EGRESS,
                managed: true,
            })
            .collect(),
        volumes: volumes.into_iter().map(managed_volume).collect(),
        images: strings(&images),
        containers,
    }
}

fn managed_volume(name: &str) -> SnapVolume {
    SnapVolume {
        name: name.to_owned(),
        managed: true,
    }
}

/// Docker in memory. It panics on a call that the real Docker would refuse, or that touches an
/// unmanaged container, network or volume.
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
                managed: true,
            });
        }
        Action::CreateVolume(spec) => {
            assert!(snapshot.volume(&spec.name).is_none());
            snapshot.volumes.push(managed_volume(&spec.name));
        }
        Action::PullImage(image) => {
            assert!(!snapshot.images.contains(image));
            snapshot.images.push(image.clone());
        }
        Action::CreateContainer(config) => {
            assert!(snapshot.container(&config.name).is_none(), "name taken");
            assert!(snapshot.images.contains(&config.image), "image missing");
            for endpoint in &config.endpoints {
                let network = snapshot.network(&endpoint.network);
                assert!(network.is_some(), "network missing");
                assert!(
                    network.is_some_and(|network| network.managed),
                    "foreign network"
                );
            }
            for mount in &config.host_config.mounts {
                let volume = snapshot.volume(&mount.source);
                assert!(volume.is_some(), "volume missing");
                assert!(
                    volume.is_some_and(|volume| volume.managed),
                    "foreign volume"
                );
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
            assert!(
                snapshot
                    .network(network)
                    .is_some_and(|network| network.managed),
                "network missing or foreign"
            );
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
            prop_assert!(snapshot.volume(&volume.name).is_some());
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
            network: NETWORKS[0].to_owned(),
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
        networks: vec![network_spec(NETWORKS[0], true)],
        volumes: Vec::new(),
        containers: vec![spec.clone()],
    }
}

fn network_spec(name: &str, internal: bool) -> NetworkSpec {
    NetworkSpec {
        name: name.to_owned(),
        internal,
        environment_id: "env".to_owned(),
    }
}

fn snap_network(name: &str, internal: bool, managed: bool) -> SnapNetwork {
    SnapNetwork {
        name: name.to_owned(),
        internal,
        managed,
    }
}

fn running(spec: &ContainerSpec, state: SnapState, started_at_ms: u64) -> DockerSnapshot {
    DockerSnapshot {
        networks: vec![snap_network(NETWORKS[0], true, true)],
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
        network: NETWORKS[0].to_owned(),
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
fn container_without_networks_is_kept_in_mode_none() {
    let mut spec = nginx(Healthcheck::Running { grace_secs: 0 });
    spec.networks.clear();
    let mut desired = one(&spec);
    desired.networks.clear();
    let mut snapshot = running(&spec, SnapState::Running, 0);
    snapshot.containers[0].networks = vec![NetworkAttachment {
        network: "none".to_owned(),
        aliases: Vec::new(),
    }];
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
        networks: vec![snap_network(NETWORKS[0], false, true)],
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
    // Containers only join networks and mount volumes that exist: the missing ones here are
    // used by no container.
    desired.networks.push(network_spec(NETWORKS[1], true));
    desired.volumes.push(VolumeSpec {
        name: VOLUMES[0].to_owned(),
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

fn creates_container(actions: &[Action]) -> bool {
    actions
        .iter()
        .any(|action| matches!(action, Action::CreateContainer(_)))
}

fn plan_at(desired: &DesiredState, snapshot: &DockerSnapshot, now_ms: u64) -> Vec<Action> {
    plan(
        desired,
        snapshot,
        &Restarts::default(),
        &ProbeResults::default(),
        now_ms,
    )
}

/// A snapshot with the image of `nginx`, these networks and volumes, and no container.
fn empty_docker(networks: Vec<SnapNetwork>, volumes: Vec<SnapVolume>) -> DockerSnapshot {
    DockerSnapshot {
        networks,
        volumes,
        images: strings(&[IMAGES[0]]),
        containers: Vec::new(),
    }
}

#[test]
fn http_health_recovers_after_an_agent_restart() {
    let spec = nginx(Healthcheck::Http {
        port: 8080,
        path: "/health".to_owned(),
        timeout_secs: 10,
    });
    let desired = one(&spec);
    let started_at_ms = 100_000;
    let snapshot = running(&spec, SnapState::Running, started_at_ms);
    let restarts = Restarts::default();
    let probe = vec![Action::Probe {
        container: spec.name.clone(),
        network: NETWORKS[0].to_owned(),
        port: 8080,
        path: "/health".to_owned(),
    }];
    let at = |probes: &ProbeResults, now_ms| {
        health(&report(&desired, &snapshot, probes, &restarts, now_ms))
    };

    // Running for hours, and the agent just restarted: no probe on record, so no verdict yet.
    let now_ms = started_at_ms + 3 * 3_600_000;
    let mut probes = ProbeResults::default();
    assert!(!probes.attempted(&spec.name, started_at_ms));
    assert_eq!(at(&probes, now_ms), ContainerHealth::Starting);
    assert_eq!(plan(&desired, &snapshot, &restarts, &probes, now_ms), probe);
    probes.record(&spec.name, started_at_ms, true, now_ms);
    assert_eq!(at(&probes, now_ms), ContainerHealth::Healthy);
    assert!(plan(&desired, &snapshot, &restarts, &probes, now_ms + 2_000).is_empty());

    // A probe that fails past the timeout still makes it unhealthy, and the probes stop.
    let mut failing = ProbeResults::default();
    failing.record(&spec.name, started_at_ms, false, now_ms);
    assert!(failing.attempted(&spec.name, started_at_ms));
    assert!(!failing.attempted(&spec.name, started_at_ms + 1));
    assert!(matches!(
        at(&failing, now_ms),
        ContainerHealth::Unhealthy { .. }
    ));
    assert!(plan(&desired, &snapshot, &restarts, &failing, now_ms + 2_000).is_empty());
}

#[test]
fn network_named_host_is_neither_created_nor_joined() {
    let mut spec = nginx(Healthcheck::Running { grace_secs: 0 });
    spec.networks[0].network = "host".to_owned();
    let mut desired = one(&spec);
    desired.networks = vec![network_spec("host", false)];
    // Docker's own networks: present, unmanaged, `host` with the internet.
    let snapshot = empty_docker(
        vec![
            snap_network("bridge", false, false),
            snap_network("host", false, false),
            snap_network("none", false, false),
        ],
        Vec::new(),
    );
    let actions = plan_at(&desired, &snapshot, 0);
    assert!(actions.is_empty(), "{actions:?}");

    // Nor when Docker does not list it.
    let actions = plan_at(&desired, &empty_docker(Vec::new(), Vec::new()), 0);
    assert!(actions.is_empty(), "{actions:?}");
}

#[test]
fn only_brume_names_are_created() {
    let bad_networks = [
        "host",
        "bridge",
        "none",
        "default",
        "container:brume-db",
        "brume-env:x",
        "Brume-env-x",
        "brume-ENV-x",
        "brume-env_x",
        "brume-env/x",
        "brume-",
        "",
    ];
    let bad_volumes = [
        "/etc",
        "data",
        "brume-vol-",
        "brume-vol-A",
        "brume-vol-x:/etc",
        "brume-env-x",
        "",
    ];
    for name in bad_networks {
        let mut spec = nginx(Healthcheck::Running { grace_secs: 0 });
        spec.networks[0].network = name.to_owned();
        let mut desired = one(&spec);
        desired.networks = vec![network_spec(name, true)];
        let actions = plan_at(&desired, &empty_docker(Vec::new(), Vec::new()), 0);
        assert!(actions.is_empty(), "network {name:?}: {actions:?}");
    }
    for name in bad_volumes {
        let mut spec = nginx(Healthcheck::Running { grace_secs: 0 });
        spec.mounts = vec![Mount {
            volume: name.to_owned(),
            path: "/data".to_owned(),
        }];
        let mut desired = one(&spec);
        desired.volumes = vec![VolumeSpec {
            name: name.to_owned(),
            service_id: "svc".to_owned(),
        }];
        let snapshot = empty_docker(vec![snap_network(NETWORKS[0], true, true)], Vec::new());
        let actions = plan_at(&desired, &snapshot, 0);
        assert!(actions.is_empty(), "volume {name:?}: {actions:?}");
    }

    // The names that the engine gives.
    let desired = DesiredState {
        position: 1,
        networks: vec![
            network_spec("brume-env-0190a3f2", true),
            network_spec("brume-egress-0190a3f2", false),
        ],
        volumes: vec![VolumeSpec {
            name: "brume-vol-0190a3f2-0".to_owned(),
            service_id: "svc".to_owned(),
        }],
        containers: Vec::new(),
    };
    let actions = plan_at(&desired, &empty_docker(Vec::new(), Vec::new()), 0);
    assert_eq!(
        actions,
        [
            Action::CreateNetwork(desired.networks[0].clone()),
            Action::CreateNetwork(desired.networks[1].clone()),
            Action::CreateVolume(desired.volumes[0].clone()),
        ]
    );
}

#[test]
fn foreign_network_with_a_brume_name_blocks_the_container() {
    let spec = nginx(Healthcheck::Running { grace_secs: 0 });
    let desired = one(&spec);
    let snapshot = empty_docker(vec![snap_network(NETWORKS[0], true, false)], Vec::new());
    let actions = plan_at(&desired, &snapshot, 0);
    assert!(actions.is_empty(), "{actions:?}");

    // A managed container is never reattached to it either.
    let mut attached = running(&spec, SnapState::Running, 0);
    attached.networks[0].managed = false;
    attached.containers[0].networks[0].aliases = strings(&["old"]);
    let actions = plan_at(&desired, &attached, 1_000);
    assert!(actions.is_empty(), "{actions:?}");
}

#[test]
fn foreign_volume_blocks_the_container() {
    let mut spec = nginx(Healthcheck::Running { grace_secs: 0 });
    spec.mounts = vec![Mount {
        volume: VOLUMES[0].to_owned(),
        path: "/data".to_owned(),
    }];
    let mut desired = one(&spec);
    desired.volumes = vec![VolumeSpec {
        name: VOLUMES[0].to_owned(),
        service_id: "svc".to_owned(),
    }];
    // Say, a `local` volume with the driver options `o=bind`: a host directory.
    let foreign = SnapVolume {
        name: VOLUMES[0].to_owned(),
        managed: false,
    };
    let snapshot = empty_docker(vec![snap_network(NETWORKS[0], true, true)], vec![foreign]);
    let actions = plan_at(&desired, &snapshot, 0);
    assert!(!creates_container(&actions), "{actions:?}");
    assert!(actions.is_empty(), "{actions:?}");
}

#[test]
fn containers_wait_for_their_networks_and_volumes() {
    let mut spec = nginx(Healthcheck::Running { grace_secs: 0 });
    spec.mounts = vec![Mount {
        volume: VOLUMES[0].to_owned(),
        path: "/data".to_owned(),
    }];
    let mut desired = one(&spec);
    desired.volumes = vec![VolumeSpec {
        name: VOLUMES[0].to_owned(),
        service_id: "svc".to_owned(),
    }];
    // Created in this round; the container joins them in the next one, once Docker lists
    // them as Brume's.
    let actions = plan_at(&desired, &empty_docker(Vec::new(), Vec::new()), 0);
    assert_eq!(
        actions,
        [
            Action::CreateNetwork(desired.networks[0].clone()),
            Action::CreateVolume(desired.volumes[0].clone()),
        ]
    );
    let created = empty_docker(
        vec![snap_network(NETWORKS[0], true, true)],
        vec![managed_volume(VOLUMES[0])],
    );
    let actions = plan_at(&desired, &created, 2_000);
    assert_eq!(
        actions,
        [
            Action::CreateContainer(Box::new(container_config(&spec))),
            Action::StartContainer(spec.name.clone()),
        ]
    );
}

#[test]
fn refusals_name_what_blocks_containers() {
    let desired = DesiredState {
        position: 1,
        networks: vec![
            network_spec("host", false),
            network_spec("brume-env-a", true),
            network_spec("brume-env-b", true),
            network_spec("brume-env-c", true),
            network_spec("brume-env-d", true),
        ],
        volumes: ["data", "brume-vol-a", "brume-vol-b", "brume-vol-c"]
            .iter()
            .map(|name| VolumeSpec {
                name: (*name).to_owned(),
                service_id: "svc".to_owned(),
            })
            .collect(),
        containers: Vec::new(),
    };
    let snapshot = empty_docker(
        vec![
            snap_network("host", false, false),
            snap_network("brume-env-a", true, false),
            snap_network("brume-env-b", false, true),
            snap_network("brume-env-d", true, true),
        ],
        vec![
            SnapVolume {
                name: "brume-vol-a".to_owned(),
                managed: false,
            },
            managed_volume("brume-vol-c"),
        ],
    );
    let refusal = |resource, name: &str, reason| Refusal {
        resource,
        name: name.to_owned(),
        reason,
    };
    assert_eq!(
        refusals(&desired, &snapshot),
        [
            refusal(Resource::Network, "host", RefusalReason::BadName),
            refusal(Resource::Network, "brume-env-a", RefusalReason::Foreign),
            refusal(
                Resource::Network,
                "brume-env-b",
                RefusalReason::WrongIsolation
            ),
            refusal(Resource::Volume, "data", RefusalReason::BadName),
            refusal(Resource::Volume, "brume-vol-a", RefusalReason::Foreign),
        ]
    );
}
