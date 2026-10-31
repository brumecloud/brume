//! The Docker actions that bring a snapshot to the desired state. `plan` is idempotent: run on
//! the snapshot that its actions produce, it returns nothing, except the probes of containers
//! still starting, every 2 s, until their health is decided.

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

/// The actions that bring `snapshot` to `desired`, in order: networks, then volumes, missing
/// images, missing containers (create, then start), restarts of stopped containers when
/// `restarts` allows, aliases, probes, and last the removal of managed containers that are not
/// wanted or whose config changed. A container without the `brume.managed` label is never
/// touched; unused networks and volumes stay.
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
        if snapshot.network(&network.name).is_none() && network_names.insert(&network.name) {
            steps.networks.push(Action::CreateNetwork(network.clone()));
        }
    }
    let mut volume_names = BTreeSet::new();
    for volume in &desired.volumes {
        if !snapshot.volumes.contains(&volume.name) && volume_names.insert(&volume.name) {
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
        if !same {
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

/// Whether Docker can create `spec`'s container safely once the networks and volumes of the
/// plan exist: each network and volume is wanted or present, and no network exists with an
/// isolation other than the wanted one, which would give the container the internet.
fn can_create(spec: &ContainerSpec, desired: &DesiredState, snapshot: &DockerSnapshot) -> bool {
    let networks_ok = spec.networks.iter().all(|attachment| {
        let wanted = desired
            .networks
            .iter()
            .find(|network| network.name == attachment.network);
        match (wanted, snapshot.network(&attachment.network)) {
            (Some(wanted), Some(actual)) => wanted.internal == actual.internal,
            (Some(_), None) => true,
            // Present but not wanted: its isolation is unknown.
            (None, _) => false,
        }
    });
    let volumes_ok = spec.mounts.iter().all(|mount| {
        snapshot.volumes.contains(&mount.volume)
            || desired
                .volumes
                .iter()
                .any(|volume| volume.name == mount.volume)
    });
    networks_ok && volumes_ok
}
