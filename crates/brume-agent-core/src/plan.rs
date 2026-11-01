//! The Docker actions that bring a snapshot to the desired state. `plan` is idempotent: run on
//! the snapshot that its actions produce, it returns nothing, except the probes of containers
//! still starting, every 2 s, until their health is decided.
//!
//! A name from the desired state reaches Docker only in the form Brume gives: `brume-[a-z0-9-]+`
//! for a network, `brume-vol-[a-z0-9-]+` for a volume. And a container only joins a network, or
//! mounts a volume, that Docker lists with the `brume.managed` label: never `host`, nor a
//! volume that someone else created under a name that Brume would give.
//!
//! Two containers never mount one volume at once: two database servers on one data directory
//! corrupt it. A container is created only once Docker lists no other container on its volumes,
//! including a container that Docker failed to inspect, and no other create of the same plan
//! takes one of them: when a deployment swaps the container of a volume, the old one goes
//! first.
//!
//! The public entry's components, from `desired.ingress`, are planned like the engine's
//! containers, with the configs of `ingress`. Their networks, `brume-tunnel` and
//! `brume-ingress`, are theirs alone: the engine can neither declare them nor attach a container
//! to them. Only they have a fixed subnet: Docker picks the engine's networks from its pools,
//! whatever subnet the engine asks for, which could cover the host's gateway or the metadata
//! endpoints.

use std::collections::BTreeSet;

use brume_proto::agent::{
    ContainerHealth, ContainerSpec, DesiredState, Healthcheck, NetworkSpec, VolumeSpec,
};

use crate::backoff::Restarts;
use crate::config::{ContainerConfig, NO_NETWORK, container_config};
use crate::health::{ProbeResults, alias_set, container_health};
use crate::ingress::{ingress_components, is_component_network};
use crate::observed::{DockerSnapshot, SnapContainer, SnapNetwork, SnapState};

/// One Docker call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Create the network, labelled `brume.managed=1`.
    CreateNetwork(NetworkSpec),
    /// Remove the network, which Docker has with another subnet than the wanted one: the plan
    /// removes its containers first, and the next one creates it again.
    RemoveNetwork(String),
    /// Create the volume, labelled `brume.managed=1`.
    CreateVolume(VolumeSpec),
    /// Pull the image, `name@sha256:…`.
    PullImage(String),
    /// Create the container; boxed, as the config is far larger than the other actions.
    CreateContainer(Box<ContainerConfig>),
    StartContainer(String),
    StopContainer {
        name: String,
        timeout_secs: u32,
    },
    /// Remove the container, even if it still runs.
    RemoveContainer(String),
    /// Disconnect the container from the network, if it is attached, then connect it with
    /// these aliases, and this fixed address if any.
    Reattach {
        container: String,
        network: String,
        aliases: Vec<String>,
        ipv4_address: Option<String>,
    },
    /// Disconnect the running container from a network that its spec no longer lists, such as
    /// the egress network: the change applies at once, without a new container.
    Disconnect {
        container: String,
        network: String,
    },
    /// `GET http://<container>:<port><path>` from a throwaway container on `network`.
    Probe {
        container: String,
        network: String,
        port: u16,
        path: String,
    },
}

/// The prefix of the names of Brume's networks.
const NETWORK_PREFIX: &str = "brume-";
/// The prefix of the names of Brume's volumes.
const VOLUME_PREFIX: &str = "brume-vol-";
/// Docker's own networks and network modes, which a Brume network never is.
const RESERVED_NETWORKS: [&str; 4] = ["host", "bridge", "none", "default"];

/// The actions that bring `snapshot` to `desired`, in order: networks, then volumes, missing
/// images, missing containers (create, then start), restarts of stopped containers when
/// `restarts` allows, aliases and networks joined, networks left, probes, the removal of
/// managed containers that are not wanted or whose config changed, and last the removal of
/// networks to create again. A container without the `brume.managed` label is never touched;
/// unused networks and volumes stay.
///
/// A missing container is created once Docker lists its networks and volumes as Brume's, so
/// one round after they are: a network or a volume of `refusals` blocks it for good. It also
/// waits until no other container mounts one of its volumes: the plan that removes the old
/// holder does not create its successor, the next one does.
///
/// A network that Docker has with another subnet than the wanted one goes, with the managed
/// containers on it: the next plans create it again, then its containers.
pub fn plan(
    desired: &DesiredState,
    snapshot: &DockerSnapshot,
    restarts: &Restarts,
    probes: &ProbeResults,
    now_ms: u64,
) -> Vec<Action> {
    let target = Target::new(desired, snapshot);
    let mut steps = Steps::default();
    let mut network_names = BTreeSet::new();
    for network in &target.networks {
        if !network_names.insert(&network.name) || network_refusal(network, snapshot).is_some() {
            continue;
        }
        match snapshot.network(&network.name) {
            None => steps.networks.push(Action::CreateNetwork(network.clone())),
            Some(actual) if !subnet_matches(network, actual) => {
                for container in snapshot.containers.iter().filter(|container| {
                    container.managed
                        && container
                            .networks
                            .iter()
                            .any(|attachment| attachment.network == network.name)
                }) {
                    steps.remove(container);
                }
                steps
                    .network_removals
                    .push(Action::RemoveNetwork(network.name.clone()));
            }
            Some(_) => {}
        }
    }
    let mut volume_names = BTreeSet::new();
    for volume in &desired.volumes {
        if snapshot.volume(&volume.name).is_none()
            && volume_refusal(&volume.name, snapshot).is_none()
            && volume_names.insert(&volume.name)
        {
            steps.volumes.push(Action::CreateVolume(volume.clone()));
        }
    }

    let mut wanted = BTreeSet::new();
    for container in &target.containers {
        if wanted.insert(container.config.name.as_str()) {
            plan_container(
                container, &target, snapshot, restarts, probes, now_ms, &mut steps,
            );
        }
    }
    for container in &snapshot.containers {
        if container.managed && !wanted.contains(container.name.as_str()) {
            steps.remove(container);
        }
    }
    steps.into_actions()
}

/// What must run: the engine's networks and containers, and the public entry's.
struct Target<'a> {
    /// The engine's networks but those of the entry, without a subnet, then the entry's.
    networks: Vec<NetworkSpec>,
    /// The entry's components first, so that an engine's container cannot take their names,
    /// then the engine's containers.
    containers: Vec<Wanted<'a>>,
}

/// A container that must run.
struct Wanted<'a> {
    config: ContainerConfig,
    /// The engine's spec, for its healthcheck; `None` for a component, which needs no probe.
    spec: Option<&'a ContainerSpec>,
}

impl Wanted<'_> {
    /// Whether the container may join `network`: an engine's container never joins one of the
    /// entry's networks, from which it would reach Traefik without Cloudflare.
    fn may_join(&self, network: &str) -> bool {
        self.spec.is_none() || !is_component_network(network)
    }
}

impl<'a> Target<'a> {
    fn new(desired: &'a DesiredState, snapshot: &DockerSnapshot) -> Self {
        let mut networks: Vec<NetworkSpec> = desired
            .networks
            .iter()
            .filter(|network| !is_component_network(&network.name))
            .map(|network| NetworkSpec {
                subnet: None,
                ..network.clone()
            })
            .collect();
        let mut containers = Vec::with_capacity(desired.containers.len() + 2);
        if let Some(ingress) = &desired.ingress {
            let (entry_networks, components) = ingress_components(ingress);
            networks.extend(entry_networks);
            for mut config in components {
                // Besides the entry's networks, a component joins the internal networks that it
                // can join now, and the others once it can: an edge network that is not there
                // yet, or refused, never holds up the entry of the other services. These
                // networks come after the first, so they are not in the config's hash.
                let edges = config.endpoints.split_off(config.endpoints.len().min(1));
                config
                    .endpoints
                    .extend(edges.into_iter().filter(|endpoint| {
                        is_component_network(&endpoint.network)
                            || (network_usable(&endpoint.network, &networks, snapshot)
                                && networks
                                    .iter()
                                    .find(|network| network.name == endpoint.network)
                                    .is_some_and(|network| network.internal))
                    }));
                containers.push(Wanted { config, spec: None });
            }
        }
        containers.extend(desired.containers.iter().map(|spec| Wanted {
            config: container_config(spec),
            spec: Some(spec),
        }));
        Self {
            networks,
            containers,
        }
    }
}

/// The actions of a plan, by step.
#[derive(Default)]
struct Steps {
    networks: Vec<Action>,
    volumes: Vec<Action>,
    pulls: Vec<Action>,
    creates: Vec<Action>,
    starts: Vec<Action>,
    reattaches: Vec<Action>,
    disconnects: Vec<Action>,
    probes: Vec<Action>,
    removals: Vec<Action>,
    network_removals: Vec<Action>,
    /// The volumes of the containers that `creates` holds.
    claimed_volumes: BTreeSet<String>,
    /// The containers that `removals` holds.
    removed: BTreeSet<String>,
}

impl Steps {
    /// Stops `container` if it runs, then removes it, once.
    fn remove(&mut self, container: &SnapContainer) {
        if !self.removed.insert(container.name.clone()) {
            return;
        }
        if container.state == SnapState::Running {
            self.removals.push(Action::StopContainer {
                name: container.name.clone(),
                timeout_secs: container.stop_timeout_secs,
            });
        }
        self.removals
            .push(Action::RemoveContainer(container.name.clone()));
    }

    fn into_actions(self) -> Vec<Action> {
        [
            self.networks,
            self.volumes,
            self.pulls,
            self.creates,
            self.starts,
            self.reattaches,
            self.disconnects,
            self.probes,
            self.removals,
            self.network_removals,
        ]
        .concat()
    }
}

/// The actions for one wanted container.
fn plan_container(
    wanted: &Wanted<'_>,
    target: &Target<'_>,
    snapshot: &DockerSnapshot,
    restarts: &Restarts,
    probes: &ProbeResults,
    now_ms: u64,
    steps: &mut Steps,
) {
    let config = &wanted.config;
    let name = &config.name;
    let Some(container) = snapshot.container(name) else {
        if can_create(wanted, &target.networks, snapshot)
            && volumes_free(config, snapshot, &steps.claimed_volumes)
        {
            steps.claimed_volumes.extend(
                config
                    .host_config
                    .mounts
                    .iter()
                    .map(|mount| mount.source.clone()),
            );
            let pull = Action::PullImage(config.image.clone());
            if !snapshot.has_image(&config.image) && !steps.pulls.contains(&pull) {
                steps.pulls.push(pull);
            }
            steps
                .creates
                .push(Action::CreateContainer(Box::new(config.clone())));
            steps.creates.push(Action::StartContainer(name.clone()));
        }
        return;
    };
    if !container.managed || steps.removed.contains(name) {
        return;
    }
    if needs_replacement(config, container) {
        // Created again at the next reconciliation, once the name is free.
        steps.remove(container);
        return;
    }

    match container.state {
        SnapState::Created | SnapState::Exited { .. } => {
            if restarts.allows_start(name, now_ms) {
                steps.starts.push(Action::StartContainer(name.clone()));
            }
        }
        SnapState::Running | SnapState::Other => {}
    }

    for endpoint in &config.endpoints {
        let actual = container
            .networks
            .iter()
            .find(|attachment| attachment.network == endpoint.network);
        let same =
            actual.is_some_and(|actual| alias_set(&actual.aliases) == alias_set(&endpoint.aliases));
        if !same
            && wanted.may_join(&endpoint.network)
            && network_usable(&endpoint.network, &target.networks, snapshot)
        {
            steps.reattaches.push(Action::Reattach {
                container: name.clone(),
                network: endpoint.network.clone(),
                aliases: endpoint.aliases.clone(),
                ipv4_address: endpoint.ipv4_address.clone(),
            });
        }
    }
    for actual in &container.networks {
        // Docker lists the `none` mode of a container without networks as an attachment.
        let wanted = actual.network == NO_NETWORK
            || config
                .endpoints
                .iter()
                .any(|endpoint| endpoint.network == actual.network);
        if !wanted {
            steps.disconnects.push(Action::Disconnect {
                container: name.clone(),
                network: actual.network.clone(),
            });
        }
    }

    let Some(spec) = wanted.spec else {
        return;
    };
    if let (Healthcheck::Http { port, path, .. }, Some(first), Some(started_at_ms)) = (
        &spec.healthcheck,
        config.endpoints.first(),
        container.started_at_ms,
    ) {
        let starting =
            container_health(spec, Some(container), probes, now_ms) == ContainerHealth::Starting;
        if container.state == SnapState::Running
            && starting
            && probes.due(name, started_at_ms, now_ms)
        {
            steps.probes.push(Action::Probe {
                container: name.clone(),
                network: first.network.clone(),
                port: *port,
                path: path.clone(),
            });
        }
    }
}

/// Whether `container` cannot become `config`'s container in place: its config changed, or
/// Docker reports it in a state the agent does not drive. Its aliases and the networks after
/// the first change in place.
fn needs_replacement(config: &ContainerConfig, container: &SnapContainer) -> bool {
    container.config_hash != config.config_hash() || container.state == SnapState::Other
}

/// Whether no container mounts one of `config`'s volumes: none that Docker lists, Brume's or
/// not, running or stopped, inspected or not, and none that this plan creates (`claimed`).
fn volumes_free(
    config: &ContainerConfig,
    snapshot: &DockerSnapshot,
    claimed: &BTreeSet<String>,
) -> bool {
    snapshot
        .containers
        .iter()
        .flat_map(|container| &container.mounts)
        .chain(&snapshot.uninspected_mounts)
        .chain(claimed)
        .all(|held| {
            config
                .host_config
                .mounts
                .iter()
                .all(|mount| mount.source != *held)
        })
}

/// Whether Docker can create `wanted`'s container safely now: it may join each of its
/// networks, which is usable, and each of its volumes is usable.
fn can_create(wanted: &Wanted<'_>, networks: &[NetworkSpec], snapshot: &DockerSnapshot) -> bool {
    let config = &wanted.config;
    let networks_usable = config.endpoints.iter().all(|endpoint| {
        wanted.may_join(&endpoint.network) && network_usable(&endpoint.network, networks, snapshot)
    });
    let volumes_usable = config
        .host_config
        .mounts
        .iter()
        .all(|mount| volume_usable(&mount.source, snapshot));
    networks_usable && volumes_usable
}

/// Whether a container may join the network `name` now: `networks` declares it, so its
/// isolation is known, and Docker lists it as Brume's, with that isolation and subnet.
fn network_usable(name: &str, networks: &[NetworkSpec], snapshot: &DockerSnapshot) -> bool {
    networks
        .iter()
        .find(|network| network.name == name)
        .is_some_and(|wanted| {
            snapshot
                .network(name)
                .is_some_and(|actual| subnet_matches(wanted, actual))
                && network_refusal(wanted, snapshot).is_none()
        })
}

/// Whether Docker has the network with its wanted subnet, if it has one.
fn subnet_matches(wanted: &NetworkSpec, actual: &SnapNetwork) -> bool {
    wanted
        .subnet
        .as_ref()
        .is_none_or(|subnet| actual.subnets.contains(subnet))
}

/// Whether a container may mount the volume `name` now: Docker lists it as Brume's.
fn volume_usable(name: &str, snapshot: &DockerSnapshot) -> bool {
    snapshot.volume(name).is_some() && volume_refusal(name, snapshot).is_none()
}

/// A kind of Docker resource that `plan` creates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Resource {
    Network,
    Volume,
}

/// Why `plan` neither creates a network or volume nor lets a container use it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefusalReason {
    /// Not a name that Brume gives.
    BadName,
    /// Docker has it without the `brume.managed` label.
    Foreign,
    /// Docker has the network with another isolation than the wanted one: a container on it
    /// could get the internet.
    WrongIsolation,
    /// The engine declares one of the public entry's networks, which only its components join.
    Reserved,
}

impl RefusalReason {
    /// The reason, for the logs.
    pub fn describe(self) -> &'static str {
        match self {
            Self::BadName => "its name is not one that Brume gives",
            Self::Foreign => "Docker has it without the brume.managed label",
            Self::WrongIsolation => "Docker has it with another internal flag",
            Self::Reserved => "only the public entry's components join it",
        }
    }
}

/// A network or volume of the desired state that `plan` neither creates nor lets a container
/// use. The containers that need it stay `Missing` until an operator fixes it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Refusal {
    pub resource: Resource,
    pub name: String,
    pub reason: RefusalReason,
}

/// The networks, then the volumes, of `desired` that `plan` refuses, and why. The networks
/// are the engine's, then the public entry's.
pub fn refusals(desired: &DesiredState, snapshot: &DockerSnapshot) -> Vec<Refusal> {
    let reserved = desired
        .networks
        .iter()
        .filter(|network| is_component_network(&network.name))
        .map(|network| Refusal {
            resource: Resource::Network,
            name: network.name.clone(),
            reason: RefusalReason::Reserved,
        });
    let target = Target::new(desired, snapshot);
    let networks = target.networks.iter().filter_map(|network| {
        network_refusal(network, snapshot).map(|reason| Refusal {
            resource: Resource::Network,
            name: network.name.clone(),
            reason,
        })
    });
    let volumes = desired.volumes.iter().filter_map(|volume| {
        volume_refusal(&volume.name, snapshot).map(|reason| Refusal {
            resource: Resource::Volume,
            name: volume.name.clone(),
            reason,
        })
    });
    reserved.chain(networks).chain(volumes).collect()
}

/// Why `plan` refuses the network `wanted`, if it does.
fn network_refusal(wanted: &NetworkSpec, snapshot: &DockerSnapshot) -> Option<RefusalReason> {
    if !is_network_name(&wanted.name) {
        return Some(RefusalReason::BadName);
    }
    let actual = snapshot.network(&wanted.name)?;
    if !actual.managed {
        Some(RefusalReason::Foreign)
    } else if actual.internal != wanted.internal {
        Some(RefusalReason::WrongIsolation)
    } else {
        None
    }
}

/// Why `plan` refuses the volume `name`, if it does.
fn volume_refusal(name: &str, snapshot: &DockerSnapshot) -> Option<RefusalReason> {
    if !is_volume_name(name) {
        return Some(RefusalReason::BadName);
    }
    match snapshot.volume(name) {
        Some(actual) if !actual.managed => Some(RefusalReason::Foreign),
        _ => None,
    }
}

/// Whether `name` is a network name that Brume gives, `brume-[a-z0-9-]+`: never one of
/// Docker's own networks, nor a `container:<id>` mode.
fn is_network_name(name: &str) -> bool {
    !RESERVED_NETWORKS.contains(&name) && !name.contains(':') && has_form(name, NETWORK_PREFIX)
}

/// Whether `name` is a volume name that Brume gives, `brume-vol-[a-z0-9-]+`: never a path.
fn is_volume_name(name: &str) -> bool {
    has_form(name, VOLUME_PREFIX)
}

/// Whether `name` is `prefix` followed by at least one of `[a-z0-9-]`.
fn has_form(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix).is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    })
}
