//! The Docker config of a container, built by allowlist: every field Brume sets is fixed here,
//! nothing is copied as is from the desired state, and every other Docker field keeps its
//! default. The shell translates it to bollard's types without adding anything.
//!
//! An app container comes from the engine's `ContainerSpec`, through `container_config`. The
//! agent's own components (Traefik and cloudflared, in `ingress`) get a user, a read-only root
//! file system, fixed addresses and, for Traefik, a read-only mount of a host directory: an app
//! container never gets them, as `container_config` sets none.

use std::collections::BTreeMap;

use brume_proto::agent::{ContainerSpec, EnvValue, EnvVar};
use secrecy::ExposeSecret;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Marks a container, network or volume that Brume manages. The agent touches nothing else.
pub const LABEL_MANAGED: &str = "brume.managed";
/// The environment of a container.
pub const LABEL_ENVIRONMENT: &str = "brume.environment";
/// The service of a container.
pub const LABEL_SERVICE: &str = "brume.service";
/// The deployment of a container.
pub const LABEL_DEPLOYMENT: &str = "brume.deployment";
/// The hash of a container's config, without secret values: a different hash replaces it.
pub const LABEL_CONFIG_HASH: &str = "brume.config-hash";
/// The component of Brume that a container runs, such as `traefik`. App containers lack it.
pub const LABEL_COMPONENT: &str = "brume.component";

/// The network mode of a container without networks: no network at all, never Docker's bridge.
pub const NO_NETWORK: &str = "none";

/// The capabilities a container keeps, after it drops them all. `SETPCAP` lets an entrypoint
/// that starts as root shrink its own bounding set while it drops root, as redis:8's `setpriv`
/// does; `no-new-privileges` still forbids gaining any.
const CAP_ADD: [&str; 9] = [
    "CHOWN",
    "DAC_OVERRIDE",
    "FOWNER",
    "FSETID",
    "KILL",
    "SETGID",
    "SETUID",
    "SETPCAP",
    "NET_BIND_SERVICE",
];

/// The most processes a container can run.
const PIDS_LIMIT: i64 = 512;

const BYTES_PER_MIB: i64 = 1024 * 1024;
const NANO_CPUS_PER_MILLI: i64 = 1_000_000;

/// A container that the agent runs for Brume itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Component {
    /// Routes each public host to its service.
    Traefik,
    /// Runs the Cloudflare tunnel, and hands every request to Traefik.
    Cloudflared,
}

impl Component {
    /// The value of its `brume.component` label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Traefik => "traefik",
            Self::Cloudflared => "cloudflared",
        }
    }
}

/// The config of a container: every Docker field that Brume sets, and nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerConfig {
    pub name: String,
    pub image: String,
    /// `Env`, as `NAME=value` at the bollard boundary only: a secret stays masked until then.
    pub env: Vec<EnvVar>,
    /// The variables of the spec left out of `env`, as their name is not
    /// `[A-Za-z_][A-Za-z0-9_]*`: Docker could refuse one and echo it, value included, in its
    /// error.
    pub skipped_env: u32,
    /// `Cmd`: the argv, or `None` to keep the image's.
    pub cmd: Option<Vec<String>>,
    /// `User`, as `uid:gid`, or `None` to keep the image's. A component's only.
    pub user: Option<String>,
    pub labels: BTreeMap<String, String>,
    /// `StopTimeout`.
    pub stop_timeout_secs: u32,
    pub host_config: HostConfig,
    /// `NetworkingConfig.EndpointsConfig`: every network, with the container's aliases on it.
    pub endpoints: Vec<Endpoint>,
    /// The component it runs, or `None` for an app container.
    pub component: Option<Component>,
}

impl ContainerConfig {
    /// The value of its `brume.config-hash` label.
    pub fn config_hash(&self) -> &str {
        self.labels
            .get(LABEL_CONFIG_HASH)
            .map_or("", String::as_str)
    }
}

/// A network in `NetworkingConfig.EndpointsConfig`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub network: String,
    pub aliases: Vec<String>,
    /// `IPAMConfig.IPv4Address`: a fixed address on the network. A component's only.
    pub ipv4_address: Option<String>,
}

/// `HostConfig`. The fields that only components set are left out of the config hash when
/// unset, so that the hash of an app container stays the same across versions of the agent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HostConfig {
    /// `NetworkMode`: the first network, or `none`.
    pub network_mode: String,
    pub cap_drop: Vec<String>,
    pub cap_add: Vec<String>,
    pub security_opt: Vec<String>,
    /// `Memory`, in bytes.
    pub memory_bytes: i64,
    /// `MemorySwap`, in bytes: equal to `Memory`, so no swap.
    pub memory_swap_bytes: i64,
    pub nano_cpus: i64,
    pub pids_limit: i64,
    pub restart_policy: RestartPolicy,
    pub log_config: LogConfig,
    /// Named volumes only.
    pub mounts: Vec<VolumeMount>,
    pub privileged: bool,
    pub publish_all_ports: bool,
    /// `ReadonlyRootfs`. A component's only.
    #[serde(skip_serializing_if = "is_false")]
    pub readonly_rootfs: bool,
    /// Read-only bind mounts of a directory that the agent owns: a component's only, and never
    /// any other directory of the host.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub binds: Vec<BindMount>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// `HostConfig.RestartPolicy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RestartPolicy {
    /// Docker never restarts the container: the agent does.
    No,
}

/// `HostConfig.LogConfig`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LogConfig {
    pub driver: String,
    pub options: BTreeMap<String, String>,
}

/// A directory of the host that a component mounts. The shell knows its path: the config
/// cannot name any other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum HostDir {
    /// Where the agent writes Traefik's config, `Paths::traefik_dir`.
    TraefikConfig,
}

/// A read-only bind mount in `HostConfig.Mounts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BindMount {
    pub source: HostDir,
    /// An absolute path in the container.
    pub target: String,
}

/// A named volume in `HostConfig.Mounts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VolumeMount {
    /// The volume's name.
    pub source: String,
    /// An absolute path in the container.
    pub target: String,
}

/// Builds the config of `spec`'s app container, field by field.
///
/// A zero CPU or memory limit becomes the smallest one, which Docker refuses, rather than
/// Docker's zero, which means no limit. A variable whose name is not a POSIX one is left out,
/// and counted in `skipped_env`.
pub fn container_config(spec: &ContainerSpec) -> ContainerConfig {
    let endpoints: Vec<Endpoint> = spec
        .networks
        .iter()
        .map(|attachment| Endpoint {
            network: attachment.network.clone(),
            aliases: attachment.aliases.clone(),
            ipv4_address: None,
        })
        .collect();
    let host_config = HostConfig {
        cap_add: CAP_ADD.iter().map(|cap| (*cap).to_owned()).collect(),
        mounts: spec
            .mounts
            .iter()
            .map(|mount| VolumeMount {
                source: mount.volume.clone(),
                target: mount.path.clone(),
            })
            .collect(),
        ..locked_host_config(&endpoints, spec.cpu_millis, spec.memory_mb)
    };
    let env: Vec<EnvVar> = spec
        .env
        .iter()
        .filter(|var| is_env_name(&var.name))
        .cloned()
        .collect();
    let skipped_env = u32::try_from(spec.env.len() - env.len()).unwrap_or(u32::MAX);
    let config = ContainerConfig {
        name: spec.name.clone(),
        image: spec.image.clone(),
        env,
        skipped_env,
        cmd: spec.command.clone(),
        user: None,
        labels: BTreeMap::from([
            (LABEL_MANAGED.to_owned(), "1".to_owned()),
            (LABEL_ENVIRONMENT.to_owned(), spec.environment_id.clone()),
            (LABEL_SERVICE.to_owned(), spec.service_id.clone()),
            (LABEL_DEPLOYMENT.to_owned(), spec.deployment_id.clone()),
        ]),
        stop_timeout_secs: spec.stop_timeout_secs,
        host_config,
        endpoints,
        component: None,
    };
    with_hash(config)
}

/// The `HostConfig` of a container on `endpoints`, with what every container of Brume shares:
/// all capabilities dropped, no new privileges, CPU, memory and pids limits, no restart policy,
/// bounded logs, no privileges, no published ports, and nothing mounted.
///
/// A zero CPU or memory limit becomes the smallest one, which Docker refuses, rather than
/// Docker's zero, which means no limit.
pub(crate) fn locked_host_config(
    endpoints: &[Endpoint],
    cpu_millis: u32,
    memory_mb: u32,
) -> HostConfig {
    let network_mode = endpoints
        .first()
        .map_or_else(|| NO_NETWORK.to_owned(), |first| first.network.clone());
    let memory_bytes = i64::from(memory_mb.max(1)) * BYTES_PER_MIB;
    HostConfig {
        network_mode,
        cap_drop: vec!["ALL".to_owned()],
        cap_add: Vec::new(),
        security_opt: vec!["no-new-privileges:true".to_owned()],
        memory_bytes,
        memory_swap_bytes: memory_bytes,
        nano_cpus: i64::from(cpu_millis.max(1)) * NANO_CPUS_PER_MILLI,
        pids_limit: PIDS_LIMIT,
        restart_policy: RestartPolicy::No,
        log_config: LogConfig {
            driver: "json-file".to_owned(),
            options: BTreeMap::from([
                ("max-file".to_owned(), "3".to_owned()),
                ("max-size".to_owned(), "10m".to_owned()),
            ]),
        },
        mounts: Vec::new(),
        privileged: false,
        publish_all_ports: false,
        readonly_rootfs: false,
        binds: Vec::new(),
    }
}

/// `config` with its `brume.config-hash` label.
pub(crate) fn with_hash(mut config: ContainerConfig) -> ContainerConfig {
    config.labels.remove(LABEL_CONFIG_HASH);
    let hash = config_hash(&config);
    config.labels.insert(LABEL_CONFIG_HASH.to_owned(), hash);
    config
}

/// Whether `name` is a POSIX variable name, `[A-Za-z_][A-Za-z0-9_]*`.
fn is_env_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// What the config hash covers: the whole config except secret values, aliases and the
/// networks after the first. Those change on a live container (`Reattach`, `Disconnect`), so
/// that egress comes and goes without a restart; the first network is the network mode, in
/// `host_config`, and everything else needs a new container. The fields that only components
/// set are left out when unset: the hash of an app container does not depend on them.
#[derive(Serialize)]
struct HashInput<'a> {
    name: &'a str,
    image: &'a str,
    env: Vec<HashedEnv<'a>>,
    cmd: Option<&'a [String]>,
    labels: &'a BTreeMap<String, String>,
    stop_timeout_secs: u32,
    host_config: &'a HostConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<&'a str>,
    /// The fixed address on each network that has one.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    addresses: Vec<(&'a str, &'a str)>,
}

/// A variable in the hash. An app's secret contributes its name only: its value must not leak
/// through a label, and a new version only reaches a container through a new deployment, whose
/// id is in the labels. A component's secret, the tunnel token, has no deployment: its SHA-256
/// stands for it, so that a new token replaces the container. The token is random and long, so
/// its hash gives nothing away, and `docker inspect` shows the token itself anyway.
#[derive(Serialize)]
enum HashedEnv<'a> {
    Plain {
        name: &'a str,
        value: &'a str,
    },
    Secret {
        name: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
    },
}

/// The SHA-256 of `config` without its hash label, in lowercase hexadecimal.
fn config_hash(config: &ContainerConfig) -> String {
    let input = HashInput {
        name: &config.name,
        image: &config.image,
        env: config
            .env
            .iter()
            .map(|var| match &var.value {
                EnvValue::Plain(value) => HashedEnv::Plain {
                    name: &var.name,
                    value,
                },
                EnvValue::Secret(secret) => HashedEnv::Secret {
                    name: &var.name,
                    sha256: config
                        .component
                        .map(|_| hex::encode(Sha256::digest(secret.expose_secret().as_bytes()))),
                },
            })
            .collect(),
        cmd: config.cmd.as_deref(),
        labels: &config.labels,
        stop_timeout_secs: config.stop_timeout_secs,
        host_config: &config.host_config,
        user: config.user.as_deref(),
        addresses: config
            .endpoints
            .iter()
            .filter_map(|endpoint| {
                endpoint
                    .ipv4_address
                    .as_deref()
                    .map(|address| (endpoint.network.as_str(), address))
            })
            .collect(),
    };
    let json = serde_json::to_vec(&input).expect("plain structs and strings always serialize");
    hex::encode(Sha256::digest(&json))
}
