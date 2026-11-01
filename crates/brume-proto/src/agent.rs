//! The messages between the engine and the agent, on `agent.sock`. The engine sends the whole
//! desired state; the agent reconciles Docker with it and reports what it observes.

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};

/// The longest failure message the agent sends, in bytes. Docker's errors are cut to it.
pub const MAX_FAILURE_MESSAGE_BYTES: usize = 1024;

/// From the engine to the agent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineToAgent {
    /// What must run. It replaces every earlier one.
    DesiredState(DesiredState),
    /// Work whose result the engine waits for.
    Job(Job),
}

/// From the agent to the engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentToEngine {
    /// The first message on each connection.
    Hello {
        /// The agent's version.
        version: String,
        /// The position of the last desired state it applied, if any.
        applied_position: Option<u64>,
    },
    /// The agent took the desired state at this position.
    Applied {
        /// The desired state's position.
        position: u64,
    },
    /// What runs, when it changes and at least every 10 seconds.
    Observed(ObservedState),
    /// The result of a `Job`.
    JobResult {
        /// The job's id.
        job_id: String,
        /// What happened.
        outcome: JobOutcome,
    },
}

/// Everything that must exist in Docker: anything else that Brume manages is removed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredState {
    /// The engine's log position when it computed this state. The agent ignores an older one.
    pub position: u64,
    pub networks: Vec<NetworkSpec>,
    pub volumes: Vec<VolumeSpec>,
    pub containers: Vec<ContainerSpec>,
    /// The public entry, once `brume install --domain` set it up.
    pub ingress: Option<IngressSpec>,
}

/// The public entry: `cloudflared` runs the tunnel with its token and hands every request to
/// Traefik, which routes each host to a service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngressSpec {
    /// Serializes its exposed value: this message only travels on `agent.sock`.
    #[serde(serialize_with = "serialize_exposed")]
    pub tunnel_token: SecretString,
    /// One per deployed service with a domain.
    pub routes: Vec<IngressRoute>,
}

impl PartialEq for IngressSpec {
    fn eq(&self, other: &Self) -> bool {
        self.tunnel_token.expose_secret() == other.tunnel_token.expose_secret()
            && self.routes == other.routes
    }
}

impl Eq for IngressSpec {}

/// A public host that Traefik sends to a service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngressRoute {
    /// `<subdomain>.<zone>`.
    pub host: String,
    /// The service's edge network, which Traefik joins.
    pub network: String,
    /// The name Traefik reaches the service by: the edge network's own name, which is also the
    /// container's alias on it. Never the slug, which another environment can share.
    pub target: String,
    /// The port the service listens on.
    pub port: u16,
}

/// A Docker network.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSpec {
    pub name: String,
    /// No route to the internet.
    pub internal: bool,
    pub environment_id: String,
    /// Its IPv4 subnet in CIDR form, or `None` to let Docker pick one from its pools. A network
    /// that Docker has with another subnet is created again.
    pub subnet: Option<String>,
}

/// A named Docker volume.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub name: String,
    pub service_id: String,
}

/// A container of a deployment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub name: String,
    /// An image pinned by digest, `name@sha256:…`.
    pub image: String,
    pub environment_id: String,
    pub service_id: String,
    pub deployment_id: String,
    pub env: Vec<EnvVar>,
    /// The argv that replaces the image's command, if any.
    pub command: Option<Vec<String>>,
    /// The first network is the container's network mode.
    pub networks: Vec<NetworkAttachment>,
    pub mounts: Vec<Mount>,
    pub cpu_millis: u32,
    pub memory_mb: u32,
    pub stop_timeout_secs: u32,
    pub healthcheck: Healthcheck,
}

/// An environment variable of a container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: EnvValue,
}

/// The value of an environment variable. A secret stays in a `SecretString` up to the Docker
/// call, so that `Debug` and `tracing` mask it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EnvValue {
    Plain(String),
    /// Serializes its exposed value: this message only travels on `agent.sock`.
    Secret(#[serde(serialize_with = "serialize_exposed")] SecretString),
}

impl PartialEq for EnvValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(left), Self::Plain(right)) => left == right,
            (Self::Secret(left), Self::Secret(right)) => {
                left.expose_secret() == right.expose_secret()
            }
            _ => false,
        }
    }
}

impl Eq for EnvValue {}

/// Serializes a secret's value, for the messages that carry one over a local socket.
pub(crate) fn serialize_exposed<S: Serializer>(
    secret: &SecretString,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(secret.expose_secret())
}

/// A network that a container joins, with its host names on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkAttachment {
    pub network: String,
    pub aliases: Vec<String>,
}

/// A named volume mounted in a container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mount {
    pub volume: String,
    /// An absolute path in the container.
    pub path: String,
}

/// How the agent decides that a starting container is healthy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Healthcheck {
    /// Healthy once it has run for `grace_secs` without exiting.
    Running { grace_secs: u32 },
    /// Healthy at the first successful `GET path` on `port`; unhealthy after `timeout_secs`.
    Http {
        port: u16,
        path: String,
        timeout_secs: u32,
    },
}

/// What the agent sees in Docker.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedState {
    pub containers: Vec<ObservedContainer>,
}

/// A container of the desired state, as Docker runs it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedContainer {
    pub name: String,
    pub deployment_id: String,
    pub health: ContainerHealth,
    /// Its host names on the environment network.
    pub aliases: Vec<String>,
    /// How many times the agent restarted it.
    pub restarts: u32,
}

/// The health of a container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerHealth {
    Starting,
    Healthy,
    Unhealthy {
        reason: String,
    },
    Exited {
        code: i64,
    },
    /// In the desired state but not in Docker.
    Missing,
}

/// Work that the engine asks of the agent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Job {
    /// Pull `image` and report its digest, volumes and ports.
    ResolveImage { job_id: String, image: String },
}

/// The result of a `Job`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobOutcome {
    ImageResolved {
        /// `name@sha256:…`.
        digest: String,
        /// The paths that the image declares as volumes.
        volumes: Vec<String>,
        exposed_ports: Vec<u16>,
    },
    /// Docker's error, cut to `MAX_FAILURE_MESSAGE_BYTES`.
    Failed { message: String },
}

#[cfg(test)]
mod tests {
    use secrecy::{ExposeSecret, SecretString};

    use super::{
        ContainerSpec, DesiredState, EnvValue, EnvVar, Healthcheck, IngressRoute, IngressSpec,
        Mount, NetworkAttachment,
    };

    #[test]
    fn secret_env_value_debug_hides_the_value() {
        let value = EnvValue::Secret(SecretString::from("s3cr3t-🚀"));
        assert!(!format!("{value:?}").contains("s3cr3t"));
    }

    #[test]
    fn secret_env_value_serializes_its_value_for_the_agent() {
        let value = EnvValue::Secret(SecretString::from("s3cr3t-🚀"));
        let json = serde_json::to_string(&value).expect("serializes");
        assert_eq!(json, r#"{"Secret":"s3cr3t-🚀"}"#);
        match serde_json::from_str::<EnvValue>(&json) {
            Ok(EnvValue::Secret(secret)) => assert_eq!(secret.expose_secret(), "s3cr3t-🚀"),
            other => panic!("expected a secret, got {other:?}"),
        }
    }

    #[test]
    fn ingress_spec_carries_its_token_to_the_agent_and_masks_it_in_debug() {
        let state = DesiredState {
            position: 7,
            networks: Vec::new(),
            volumes: Vec::new(),
            containers: Vec::new(),
            ingress: Some(IngressSpec {
                tunnel_token: SecretString::from("s3cr3t-🚀"),
                routes: vec![IngressRoute {
                    host: "demo.example.com".to_owned(),
                    network: "brume-edge-svc".to_owned(),
                    target: "web".to_owned(),
                    port: 8080,
                }],
            }),
        };
        assert!(!format!("{state:?}").contains("s3cr3t"));
        let json = serde_json::to_string(&state).expect("serializes");
        assert!(json.contains("s3cr3t-🚀"), "{json}");
        assert_eq!(
            serde_json::from_str::<DesiredState>(&json).ok(),
            Some(state.clone())
        );
        let mut other = state.clone();
        if let Some(ingress) = other.ingress.as_mut() {
            ingress.tunnel_token = SecretString::from("rotated");
        }
        assert_ne!(other, state);
    }

    #[test]
    fn container_spec_round_trips() {
        let spec = ContainerSpec {
            name: "brume-api-0190a3f2".to_owned(),
            image: format!("nginx@sha256:{}", "0".repeat(64)),
            environment_id: "env".to_owned(),
            service_id: "svc".to_owned(),
            deployment_id: "dep".to_owned(),
            env: vec![
                EnvVar {
                    name: "GREETING".to_owned(),
                    value: EnvValue::Plain("a\nb".to_owned()),
                },
                EnvVar {
                    name: "TOKEN".to_owned(),
                    value: EnvValue::Secret("s3cr3t".into()),
                },
            ],
            command: Some(vec![
                "nginx".to_owned(),
                "-g".to_owned(),
                "daemon off;".to_owned(),
            ]),
            networks: vec![NetworkAttachment {
                network: "brume-env-0190a3f2".to_owned(),
                aliases: vec!["api".to_owned()],
            }],
            mounts: vec![Mount {
                volume: "brume-vol-0190a3f2-0".to_owned(),
                path: "/data".to_owned(),
            }],
            cpu_millis: 1000,
            memory_mb: 512,
            stop_timeout_secs: 60,
            healthcheck: Healthcheck::Http {
                port: 80,
                path: "/".to_owned(),
                timeout_secs: 60,
            },
        };
        let json = serde_json::to_vec(&spec).expect("serializes");
        assert_eq!(
            serde_json::from_slice::<ContainerSpec>(&json).ok(),
            Some(spec)
        );
    }
}
