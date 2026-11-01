//! What the agent sees in Docker: the shell lists networks, volumes, images and containers, and
//! hands the cores this snapshot.

use brume_proto::agent::NetworkAttachment;

/// Everything the agent saw in Docker at one instant.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DockerSnapshot {
    pub networks: Vec<SnapNetwork>,
    pub volumes: Vec<SnapVolume>,
    /// The digest references of the images present, `name@sha256:…`.
    pub images: Vec<String>,
    pub containers: Vec<SnapContainer>,
    /// The named volumes of the managed containers that Docker listed but failed to inspect,
    /// which are left out of `containers`. Taken from the listing: no container is created on
    /// them, so a transient inspect error never lets two containers share a volume.
    pub uninspected_mounts: Vec<String>,
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

    /// The volume named `name`, if any.
    pub fn volume(&self, name: &str) -> Option<&SnapVolume> {
        self.volumes.iter().find(|volume| volume.name == name)
    }

    /// Whether Docker has the image `reference`, `name@sha256:…` or `name:tag@sha256:…`. Docker
    /// lists an image by its repository and digest only: a tag beside the digest is left out.
    pub fn has_image(&self, reference: &str) -> bool {
        let wanted = without_tag(reference);
        self.images.iter().any(|image| without_tag(image) == wanted)
    }
}

/// A digest reference without the tag of its last path component: `traefik:v3@sha256:…` gives
/// `traefik@sha256:…`. A registry's port, before a `/`, stays. Any other reference is returned
/// as is.
fn without_tag(reference: &str) -> String {
    let Some((name, digest)) = reference.split_once('@') else {
        return reference.to_owned();
    };
    let start = name.rfind('/').map_or(0, |slash| slash + 1);
    let repository = match name[start..].find(':') {
        Some(colon) => &name[..start + colon],
        None => name,
    };
    format!("{repository}@{digest}")
}

/// A Docker network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapNetwork {
    pub name: String,
    /// No route to the internet.
    pub internal: bool,
    /// It has the label `brume.managed=1`. No container of Brume joins a network without it.
    pub managed: bool,
    /// The subnets of its IPAM config, in CIDR form.
    pub subnets: Vec<String>,
}

/// A Docker volume.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapVolume {
    pub name: String,
    /// It has the label `brume.managed=1`. No container of Brume mounts a volume without it.
    pub managed: bool,
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
    /// The named volumes it mounts, whether it runs or not.
    pub mounts: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::DockerSnapshot;

    #[test]
    fn an_image_matches_its_digest_with_or_without_a_tag() {
        let snapshot = DockerSnapshot {
            images: vec![
                "traefik@sha256:aa".to_owned(),
                "localhost:5000/team/app@sha256:bb".to_owned(),
            ],
            ..DockerSnapshot::default()
        };
        assert!(snapshot.has_image("traefik@sha256:aa"));
        assert!(snapshot.has_image("traefik:v3.7.17@sha256:aa"));
        assert!(snapshot.has_image("localhost:5000/team/app:1.0@sha256:bb"));
        assert!(snapshot.has_image("localhost:5000/team/app@sha256:bb"));
        // Another digest, another repository, or no digest at all: missing.
        assert!(!snapshot.has_image("traefik:v3.7.17@sha256:cc"));
        assert!(!snapshot.has_image("other:v3@sha256:aa"));
        assert!(!snapshot.has_image("localhost:5000/team/app:1.0"));
        assert!(!snapshot.has_image("traefik"));
    }
}
