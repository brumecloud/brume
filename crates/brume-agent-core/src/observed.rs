//! What the agent sees in Docker: the shell lists networks, volumes, images and containers, and
//! hands the cores this snapshot.

use brume_proto::agent::NetworkAttachment;

/// Everything the agent saw in Docker at one instant.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DockerSnapshot {
    pub networks: Vec<SnapNetwork>,
    /// The names of the volumes.
    pub volumes: Vec<String>,
    /// The digest references of the images present, `name@sha256:…`.
    pub images: Vec<String>,
    pub containers: Vec<SnapContainer>,
}

impl DockerSnapshot {
    /// The container named `name`, if any.
    pub fn container(&self, name: &str) -> Option<&SnapContainer> {
        self.containers
            .iter()
            .find(|container| container.name == name)
    }

    /// The network named `name`, if any.
    pub fn network(&self, name: &str) -> Option<&SnapNetwork> {
        self.networks.iter().find(|network| network.name == name)
    }
}

/// A Docker network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapNetwork {
    pub name: String,
    /// No route to the internet.
    pub internal: bool,
}

/// A Docker container, managed by Brume or not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapContainer {
    pub name: String,
    /// It has the label `brume.managed=1`. The agent never touches a container without it.
    pub managed: bool,
    /// Its `brume.deployment` label, empty without one.
    pub deployment_id: String,
    /// The image it was created from.
    pub image: String,
    pub state: SnapState,
    /// When it last started, in milliseconds since the Unix epoch.
    pub started_at_ms: Option<u64>,
    /// The networks it is attached to, with its aliases on each.
    pub networks: Vec<NetworkAttachment>,
    /// Its `brume.config-hash` label, empty without one.
    pub config_hash: String,
    /// How long Docker waits after `SIGTERM` before it kills it.
    pub stop_timeout_secs: u32,
}

/// The state of a container, as Docker reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapState {
    /// Created, never started.
    Created,
    Running,
    Exited {
        code: i64,
    },
    /// Paused, restarting, being removed or dead.
    Other,
}
