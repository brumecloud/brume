//! The Docker config of a container, built by allowlist: every field Brume sets is fixed here,
//! nothing is copied as is from the desired state, and every other Docker field keeps its
//! default. The shell translates it to bollard's types without adding anything.

use std::collections::BTreeMap;

use brume_proto::agent::{ContainerSpec, EnvValue, EnvVar, NetworkAttachment};
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

/// The network mode of a container without networks: no network at all, never Docker's bridge.
pub const NO_NETWORK: &str = "none";

/// The capabilities a container keeps, after it drops them all.
const CAP_ADD: [&str; 8] = [
    "CHOWN",
    "DAC_OVERRIDE",
    "FOWNER",
    "FSETID",
    "KILL",
    "SETGID",
    "SETUID",
    "NET_BIND_SERVICE",
];

/// The most processes a container can run.
const PIDS_LIMIT: i64 = 512;

const BYTES_PER_MIB: i64 = 1024 * 1024;
const NANO_CPUS_PER_MILLI: i64 = 1_000_000;

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
    pub labels: BTreeMap<String, String>,
    /// `StopTimeout`.
    pub stop_timeout_secs: u32,
    pub host_config: HostConfig,
    /// `NetworkingConfig.EndpointsConfig`: every network, with the container's aliases on it.
    pub endpoints: Vec<NetworkAttachment>,
}

impl ContainerConfig {
    /// The value of its `brume.config-hash` label.
    pub fn config_hash(&self) -> &str {
        self.labels
            .get(LABEL_CONFIG_HASH)
            .map_or("", String::as_str)
    }
}

/// `HostConfig`.
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
    /// Named volumes only: never a bind mount of the host.
    pub mounts: Vec<VolumeMount>,
    pub privileged: bool,
    pub publish_all_ports: bool,
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

/// A named volume in `HostConfig.Mounts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VolumeMount {
    /// The volume's name.
    pub source: String,
    /// An absolute path in the container.
    pub target: String,
}

/// Builds the config of `spec`'s container, field by field.
///
/// A zero CPU or memory limit becomes the smallest one, which Docker refuses, rather than
/// Docker's zero, which means no limit. A variable whose name is not a POSIX one is left out,
/// and counted in `skipped_env`.
pub fn container_config(spec: &ContainerSpec) -> ContainerConfig {
    let network_mode = spec
        .networks
        .first()
        .map_or_else(|| NO_NETWORK.to_owned(), |first| first.network.clone());
    let memory_bytes = i64::from(spec.memory_mb.max(1)) * BYTES_PER_MIB;
    let host_config = HostConfig {
        network_mode,
        cap_drop: vec!["ALL".to_owned()],
        cap_add: CAP_ADD.iter().map(|cap| (*cap).to_owned()).collect(),
        security_opt: vec!["no-new-privileges:true".to_owned()],
        memory_bytes,
        memory_swap_bytes: memory_bytes,
        nano_cpus: i64::from(spec.cpu_millis.max(1)) * NANO_CPUS_PER_MILLI,
        pids_limit: PIDS_LIMIT,
        restart_policy: RestartPolicy::No,
        log_config: LogConfig {
            driver: "json-file".to_owned(),
            options: BTreeMap::from([
                ("max-file".to_owned(), "3".to_owned()),
                ("max-size".to_owned(), "10m".to_owned()),
            ]),
        },
        mounts: spec
            .mounts
            .iter()
            .map(|mount| VolumeMount {
                source: mount.volume.clone(),
                target: mount.path.clone(),
            })
            .collect(),
        privileged: false,
        publish_all_ports: false,
    };
    let env: Vec<EnvVar> = spec
        .env
        .iter()
        .filter(|var| is_env_name(&var.name))
        .cloned()
        .collect();
    let skipped_env = u32::try_from(spec.env.len() - env.len()).unwrap_or(u32::MAX);
    let mut config = ContainerConfig {
        name: spec.name.clone(),
        image: spec.image.clone(),
        env,
        skipped_env,
        cmd: spec.command.clone(),
        labels: BTreeMap::from([
            (LABEL_MANAGED.to_owned(), "1".to_owned()),
            (LABEL_ENVIRONMENT.to_owned(), spec.environment_id.clone()),
            (LABEL_SERVICE.to_owned(), spec.service_id.clone()),
            (LABEL_DEPLOYMENT.to_owned(), spec.deployment_id.clone()),
        ]),
        stop_timeout_secs: spec.stop_timeout_secs,
        host_config,
        endpoints: spec.networks.clone(),
    };
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

/// What the config hash covers: the whole config except secret values and aliases. Aliases
/// change on a live container (`Reattach`); everything else needs a new container.
#[derive(Serialize)]
struct HashInput<'a> {
    name: &'a str,
    image: &'a str,
    env: Vec<HashedEnv<'a>>,
    cmd: Option<&'a [String]>,
    labels: &'a BTreeMap<String, String>,
    stop_timeout_secs: u32,
    host_config: &'a HostConfig,
    networks: Vec<&'a str>,
}

/// A variable in the hash. A secret contributes its name only: its value must not leak through
/// a label, and a new version only reaches a container through a new deployment, whose id is in
/// the labels.
#[derive(Serialize)]
enum HashedEnv<'a> {
    Plain { name: &'a str, value: &'a str },
    Secret { name: &'a str },
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
                EnvValue::Secret(_) => HashedEnv::Secret { name: &var.name },
            })
            .collect(),
        cmd: config.cmd.as_deref(),
        labels: &config.labels,
        stop_timeout_secs: config.stop_timeout_secs,
        host_config: &config.host_config,
        networks: config
            .endpoints
            .iter()
            .map(|endpoint| endpoint.network.as_str())
            .collect(),
    };
    let json = serde_json::to_vec(&input).expect("plain structs and strings always serialize");
    hex::encode(Sha256::digest(&json))
}
