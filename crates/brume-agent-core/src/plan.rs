//! The Docker actions that bring a snapshot to the desired state. `plan` is idempotent: run on
//! the snapshot that its actions produce, it returns nothing, except the probes of containers
//! still starting, every 2 s, until their health is decided.
//!
//! A name from the desired state reaches Docker only in the form Brume gives: `brume-[a-z0-9-]+`
//! for a network, `brume-vol-[a-z0-9-]+` for a volume. And a container only joins a network, or
//! mounts a volume, that Docker lists with the `brume.managed` label: never `host`, nor a
//! volume that someone else created under a name that Brume would give.

use std::collections::BTreeSet;

use brume_proto::agent::{
    ContainerHealth, ContainerSpec, DesiredState, Healthcheck, NetworkSpec, VolumeSpec,
};

use crate::backoff::Restarts;
use crate::config::{ContainerConfig, NO_NETWORK, container_config};
use crate::health::{ProbeResults, alias_set, container_health};
use crate::observed::{DockerSnapshot, SnapContainer, SnapState};

/// One Docker call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Create the network, labelled `brume.managed=1`.
    CreateNetwork(NetworkSpec),
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
    /// these aliases.
    Reattach {
        container: String,
        network: String,
        aliases: Vec<String>,
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
/// `restarts` allows, aliases, probes, and last the removal of managed containers that are not
/// wanted or whose config changed. A container without the `brume.managed` label is never
/// touched; unused networks and volumes stay.
///
/// A missing container is created once Docker lists its networks and volumes as Brume's, so
/// one round after they are: a network or a volume of `refusals` blocks it for good.
pub fn plan(
    desired: &DesiredState,
    snapshot: &DockerSnapshot,
    restarts: &Restarts,
    probes: &ProbeResults,
    now_ms: u64,
) -> Vec<Action> {
    let mut steps = Steps::default();
    let mut network_names = BTreeSet::new();
    for network in &desired.networks {
        if snapshot.network(&network.name).is_none()
            && network_refusal(network, snapshot).is_none()
            && network_names.insert(&network.name)
        {
            steps.networks.push(Action::CreateNetwork(network.clone()));
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
    for spec in &desired.containers {
        if wanted.insert(spec.name.as_str()) {
            plan_container(
                spec, desired, snapshot, restarts, probes, now_ms, &mut steps,
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

/// The actions of a plan, by step.
#[derive(Default)]
struct Steps {
    networks: Vec<Action>,
    volumes: Vec<Action>,
    pulls: Vec<Action>,
    creates: Vec<Action>,
    starts: Vec<Action>,
    reattaches: Vec<Action>,
    probes: Vec<Action>,
    removals: Vec<Action>,
}

impl Steps {
    /// Stops `container` if it runs, then removes it.
    fn remove(&mut self, container: &SnapContainer) {
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
            self.probes,
            self.removals,
        ]
        .concat()
    }
}

/// The actions for one wanted container.
fn plan_container(
    spec: &ContainerSpec,
    desired: &DesiredState,
    snapshot: &DockerSnapshot,
    restarts: &Restarts,
    probes: &ProbeResults,
    now_ms: u64,
    steps: &mut Steps,
) {
    let Some(container) = snapshot.container(&spec.name) else {
        if can_create(spec, desired, snapshot) {
            let pull = Action::PullImage(spec.image.clone());
            if !snapshot.images.contains(&spec.image) && !steps.pulls.contains(&pull) {
                steps.pulls.push(pull);
            }
            steps
                .creates
                .push(Action::CreateContainer(Box::new(container_config(spec))));
            steps
                .creates
                .push(Action::StartContainer(spec.name.clone()));
        }
        return;
    };
    if !container.managed {
        return;
    }
    if needs_replacement(spec, container) {
        // Created again at the next reconciliation, once the name is free.
        steps.remove(container);
        return;
    }

    match container.state {
        SnapState::Created | SnapState::Exited { .. } => {
            if restarts.allows_start(&spec.name, now_ms) {
                steps.starts.push(Action::StartContainer(spec.name.clone()));
            }
        }
        SnapState::Running | SnapState::Other => {}
    }

    for wanted in &spec.networks {
        let actual = container
            .networks
            .iter()
            .find(|attachment| attachment.network == wanted.network);
        let same =
            actual.is_some_and(|actual| alias_set(&actual.aliases) == alias_set(&wanted.aliases));
        if !same && network_usable(&wanted.network, desired, snapshot) {
            steps.reattaches.push(Action::Reattach {
                container: spec.name.clone(),
                network: wanted.network.clone(),
                aliases: wanted.aliases.clone(),
            });
        }
    }

    if let (Healthcheck::Http { port, path, .. }, Some(first), Some(started_at_ms)) = (
        &spec.healthcheck,
        spec.networks.first(),
        container.started_at_ms,
    ) {
        let starting =
            container_health(spec, Some(container), probes, now_ms) == ContainerHealth::Starting;
        if container.state == SnapState::Running
            && starting
            && probes.due(&spec.name, started_at_ms, now_ms)
        {
            steps.probes.push(Action::Probe {
                container: spec.name.clone(),
                network: first.network.clone(),
                port: *port,
                path: path.clone(),
            });
        }
    }
}

/// Whether `container` cannot become `spec`'s container in place: its config changed, Docker
/// reports it in a state the agent does not drive, or it is attached to a network that `spec`
/// does not list.
fn needs_replacement(spec: &ContainerSpec, container: &SnapContainer) -> bool {
    container.config_hash != container_config(spec).config_hash()
        || container.state == SnapState::Other
        || container.networks.iter().any(|actual| {
            // Docker lists the `none` mode of a container without networks as an attachment.
            actual.network != NO_NETWORK
                && !spec
                    .networks
                    .iter()
                    .any(|wanted| wanted.network == actual.network)
        })
}

/// Whether Docker can create `spec`'s container safely now: each of its networks and volumes
/// is usable.
fn can_create(spec: &ContainerSpec, desired: &DesiredState, snapshot: &DockerSnapshot) -> bool {
    spec.networks
        .iter()
        .all(|attachment| network_usable(&attachment.network, desired, snapshot))
        && spec
            .mounts
            .iter()
            .all(|mount| volume_usable(&mount.volume, snapshot))
}

/// Whether a container may join the network `name` now: the desired state declares it, so its
/// isolation is known, and Docker lists it as Brume's, with that isolation.
fn network_usable(name: &str, desired: &DesiredState, snapshot: &DockerSnapshot) -> bool {
    desired
        .networks
        .iter()
        .find(|network| network.name == name)
        .is_some_and(|wanted| {
            snapshot.network(name).is_some() && network_refusal(wanted, snapshot).is_none()
        })
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
}

impl RefusalReason {
    /// The reason, for the logs.
    pub fn describe(self) -> &'static str {
        match self {
            Self::BadName => "its name is not one that Brume gives",
            Self::Foreign => "Docker has it without the brume.managed label",
            Self::WrongIsolation => "Docker has it with another internal flag",
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

/// The networks, then the volumes, of `desired` that `plan` refuses, and why.
pub fn refusals(desired: &DesiredState, snapshot: &DockerSnapshot) -> Vec<Refusal> {
    let networks = desired.networks.iter().filter_map(|network| {
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
    networks.chain(volumes).collect()
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
