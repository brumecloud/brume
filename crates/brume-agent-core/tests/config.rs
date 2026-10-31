//! The complete config of an app container, and its hash.

use std::collections::BTreeMap;

use brume_agent_core::config::{
    ContainerConfig, HostConfig, LogConfig, RestartPolicy, VolumeMount, container_config,
};
use brume_proto::agent::{ContainerSpec, EnvValue, EnvVar, Healthcheck, Mount, NetworkAttachment};
use secrecy::SecretString;

const IMAGE: &str = "nginx@sha256:5e6a1b5b6a2b8b5b4c1f6e1f0c5a0e7d2f2b1a4c3d5e6f708192a3b4c5d6e7f8";

fn env(name: &str, value: EnvValue) -> EnvVar {
    EnvVar {
        name: name.to_owned(),
        value,
    }
}

fn plain(value: &str) -> EnvValue {
    EnvValue::Plain(value.to_owned())
}

fn secret(value: &str) -> EnvValue {
    EnvValue::Secret(SecretString::from(value))
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

/// nginx with two variables, a secret, egress and a volume.
fn nginx() -> ContainerSpec {
    ContainerSpec {
        name: "brume-web-0190a3f2".to_owned(),
        image: IMAGE.to_owned(),
        environment_id: "env-1".to_owned(),
        service_id: "svc-1".to_owned(),
        deployment_id: "dep-1".to_owned(),
        env: vec![
            env("GREETING", plain("hello")),
            env("PORT", plain("80")),
            env("API_TOKEN", secret("s3cr3t-value")),
        ],
        command: None,
        networks: vec![
            NetworkAttachment {
                network: "brume-env-1".to_owned(),
                aliases: strings(&["web"]),
            },
            NetworkAttachment {
                network: "brume-egress-1".to_owned(),
                aliases: Vec::new(),
            },
        ],
        mounts: vec![Mount {
            volume: "brume-vol-svc-1-0".to_owned(),
            path: "/var/cache/nginx".to_owned(),
        }],
        cpu_millis: 500,
        memory_mb: 256,
        stop_timeout_secs: 30,
        healthcheck: Healthcheck::Http {
            port: 80,
            path: "/".to_owned(),
            timeout_secs: 60,
        },
    }
}

#[test]
fn full_config_of_an_app_container() {
    let expected = ContainerConfig {
        name: "brume-web-0190a3f2".to_owned(),
        image: IMAGE.to_owned(),
        env: vec![
            env("GREETING", plain("hello")),
            env("PORT", plain("80")),
            env("API_TOKEN", secret("s3cr3t-value")),
        ],
        cmd: None,
        labels: BTreeMap::from([
            // Pinned: a change of the hash input replaces every running container.
            (
                "brume.config-hash".to_owned(),
                "871a9acaad1054b545f999ce66b08dfe7643a946ec381cc1a30a1eb5de6141f0".to_owned(),
            ),
            ("brume.deployment".to_owned(), "dep-1".to_owned()),
            ("brume.environment".to_owned(), "env-1".to_owned()),
            ("brume.managed".to_owned(), "1".to_owned()),
            ("brume.service".to_owned(), "svc-1".to_owned()),
        ]),
        stop_timeout_secs: 30,
        host_config: HostConfig {
            network_mode: "brume-env-1".to_owned(),
            cap_drop: strings(&["ALL"]),
            cap_add: strings(&[
                "CHOWN",
                "DAC_OVERRIDE",
                "FOWNER",
                "FSETID",
                "KILL",
                "SETGID",
                "SETUID",
                "NET_BIND_SERVICE",
            ]),
            security_opt: strings(&["no-new-privileges:true"]),
            memory_bytes: 256 * 1024 * 1024,
            memory_swap_bytes: 256 * 1024 * 1024,
            nano_cpus: 500_000_000,
            pids_limit: 512,
            restart_policy: RestartPolicy::No,
            log_config: LogConfig {
                driver: "json-file".to_owned(),
                options: BTreeMap::from([
                    ("max-file".to_owned(), "3".to_owned()),
                    ("max-size".to_owned(), "10m".to_owned()),
                ]),
            },
            mounts: vec![VolumeMount {
                source: "brume-vol-svc-1-0".to_owned(),
                target: "/var/cache/nginx".to_owned(),
            }],
            privileged: false,
            publish_all_ports: false,
        },
        endpoints: vec![
            NetworkAttachment {
                network: "brume-env-1".to_owned(),
                aliases: strings(&["web"]),
            },
            NetworkAttachment {
                network: "brume-egress-1".to_owned(),
                aliases: Vec::new(),
            },
        ],
    };
    assert_eq!(container_config(&nginx()), expected);
}

#[test]
fn command_replaces_the_image_cmd() {
    let mut spec = nginx();
    spec.command = Some(strings(&["nginx", "-g", "daemon off;"]));
    assert_eq!(
        container_config(&spec).cmd,
        Some(strings(&["nginx", "-g", "daemon off;"]))
    );
}

#[test]
fn container_without_network_gets_none_never_the_bridge() {
    let mut spec = nginx();
    spec.networks.clear();
    let config = container_config(&spec);
    assert_eq!(config.host_config.network_mode, "none");
    assert!(config.endpoints.is_empty());
}

#[test]
fn zero_limits_never_mean_unlimited() {
    let mut spec = nginx();
    spec.cpu_millis = 0;
    spec.memory_mb = 0;
    let host_config = container_config(&spec).host_config;
    assert!(host_config.nano_cpus > 0);
    assert!(host_config.memory_bytes > 0);
    assert_eq!(host_config.memory_swap_bytes, host_config.memory_bytes);
}

#[test]
fn config_hash_ignores_secret_values_but_not_versions() {
    let base = container_config(&nginx());
    let hash = base.config_hash().to_owned();
    assert_eq!(hash.len(), 64);
    assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));

    // A new secret value alone keeps the container.
    let mut rotated = nginx();
    rotated.env[2] = env("API_TOKEN", secret("another-value"));
    assert_eq!(container_config(&rotated).config_hash(), hash);

    // A new secret version only reaches a container through a new deployment, which changes
    // the hash. So do a renamed secret, and a secret that becomes plain.
    let mut redeployed = nginx();
    redeployed.deployment_id = "dep-2".to_owned();
    assert_ne!(container_config(&redeployed).config_hash(), hash);

    let mut renamed = nginx();
    renamed.env[2] = env("API_KEY", secret("s3cr3t-value"));
    assert_ne!(container_config(&renamed).config_hash(), hash);

    let mut unveiled = nginx();
    unveiled.env[2] = env("API_TOKEN", plain("s3cr3t-value"));
    assert_ne!(container_config(&unveiled).config_hash(), hash);

    // Plain values, networks and limits are in it; aliases are not, the agent reattaches.
    let mut changed = nginx();
    changed.env[0] = env("GREETING", plain("bonjour"));
    assert_ne!(container_config(&changed).config_hash(), hash);

    let mut isolated = nginx();
    isolated.networks.pop();
    assert_ne!(container_config(&isolated).config_hash(), hash);

    let mut bigger = nginx();
    bigger.memory_mb = 512;
    assert_ne!(container_config(&bigger).config_hash(), hash);

    let mut renamed_alias = nginx();
    renamed_alias.networks[0].aliases = strings(&["www"]);
    assert_eq!(container_config(&renamed_alias).config_hash(), hash);
}

#[test]
fn config_debug_hides_secret_values() {
    let debug = format!("{:?}", container_config(&nginx()));
    assert!(!debug.contains("s3cr3t-value"));
    assert!(debug.contains("hello"));
}
