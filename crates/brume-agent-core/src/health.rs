//! The health of the containers of the desired state, and the HTTP probes that decide it.

use std::collections::{BTreeMap, BTreeSet};

use brume_proto::agent::{
    ContainerHealth, ContainerSpec, DesiredState, Healthcheck, ObservedContainer, ObservedState,
};

use crate::backoff::Restarts;
use crate::config::container_config;
use crate::observed::{DockerSnapshot, SnapContainer, SnapState};

/// The shortest interval between two probes of one container.
pub const PROBE_INTERVAL_MS: u64 = 2_000;

/// The probes of each container since its last start, kept by the agent across reconciliations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProbeResults {
    entries: BTreeMap<String, Probes>,
}

/// The probes of one container since one start.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Probes {
    /// The start that these probes follow.
    started_at_ms: u64,
    last_attempt_ms: u64,
    last_success_ms: Option<u64>,
    last_failure_ms: Option<u64>,
}

impl ProbeResults {
    /// Records a probe of `container`, which last started at `started_at_ms`. The probes of an
    /// earlier start are forgotten.
    pub fn record(&mut self, container: &str, started_at_ms: u64, success: bool, now_ms: u64) {
        let probes = self.entries.entry(container.to_owned()).or_default();
        if probes.started_at_ms != started_at_ms {
            *probes = Probes {
                started_at_ms,
                ..Probes::default()
            };
        }
        probes.last_attempt_ms = now_ms;
        if success {
            probes.last_success_ms = Some(now_ms);
        } else {
            probes.last_failure_ms = Some(now_ms);
        }
    }

    /// Whether a probe of `container` succeeded since its start at `started_at_ms`.
    pub fn succeeded(&self, container: &str, started_at_ms: u64) -> bool {
        self.since(container, started_at_ms)
            .is_some_and(|probes| probes.last_success_ms.is_some())
    }

    /// Whether `container`, started at `started_at_ms`, may be probed at `now_ms`: once every
    /// `PROBE_INTERVAL_MS`.
    pub fn due(&self, container: &str, started_at_ms: u64, now_ms: u64) -> bool {
        self.since(container, started_at_ms)
            .is_none_or(|probes| now_ms.saturating_sub(probes.last_attempt_ms) >= PROBE_INTERVAL_MS)
    }

    /// Forgets the containers that are gone.
    pub fn settle(&mut self, snapshot: &DockerSnapshot) {
        self.entries
            .retain(|name, _| snapshot.container(name).is_some());
    }

    /// The probes of `container` since its start at `started_at_ms`.
    fn since(&self, container: &str, started_at_ms: u64) -> Option<&Probes> {
        self.entries
            .get(container)
            .filter(|probes| probes.started_at_ms == started_at_ms)
    }
}

/// What runs, for each container of the desired state, in its order.
pub fn report(
    desired: &DesiredState,
    snapshot: &DockerSnapshot,
    probes: &ProbeResults,
    restarts: &Restarts,
    now_ms: u64,
) -> ObservedState {
    let containers = desired
        .containers
        .iter()
        .map(|spec| {
            let container = snapshot.container(&spec.name);
            let aliases = match (spec.networks.first(), container) {
                (Some(first), Some(container)) => container
                    .networks
                    .iter()
                    .find(|attachment| attachment.network == first.network)
                    .map(|attachment| alias_set(&attachment.aliases).into_iter().collect())
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            ObservedContainer {
                name: spec.name.clone(),
                deployment_id: spec.deployment_id.clone(),
                health: container_health(spec, container, probes, now_ms),
                aliases,
                restarts: restarts.restarts(&spec.name),
            }
        })
        .collect();
    ObservedState { containers }
}

/// The health of `spec`'s container, given what Docker runs under its name.
pub(crate) fn container_health(
    spec: &ContainerSpec,
    container: Option<&SnapContainer>,
    probes: &ProbeResults,
    now_ms: u64,
) -> ContainerHealth {
    let Some(container) = container else {
        return ContainerHealth::Missing;
    };
    if !container.managed {
        return ContainerHealth::Unhealthy {
            reason: "a container that Brume does not manage has its name".to_owned(),
        };
    }
    if container.config_hash != container_config(spec).config_hash() {
        // Docker still runs an older config, about to be replaced.
        return ContainerHealth::Starting;
    }
    let started_at_ms = match container.state {
        SnapState::Created => return ContainerHealth::Starting,
        SnapState::Exited { code } => return ContainerHealth::Exited { code },
        SnapState::Other => {
            return ContainerHealth::Unhealthy {
                reason: "Docker reports it paused, dead or being removed".to_owned(),
            };
        }
        SnapState::Running => match container.started_at_ms {
            Some(started_at_ms) => started_at_ms,
            None => return ContainerHealth::Starting,
        },
    };
    let running_ms = now_ms.saturating_sub(started_at_ms);
    match &spec.healthcheck {
        Healthcheck::Running { grace_secs } => {
            if running_ms >= secs_to_ms(*grace_secs) {
                ContainerHealth::Healthy
            } else {
                ContainerHealth::Starting
            }
        }
        Healthcheck::Http { timeout_secs, .. } => {
            if probes.succeeded(&spec.name, started_at_ms) {
                ContainerHealth::Healthy
            } else if running_ms >= secs_to_ms(*timeout_secs) {
                ContainerHealth::Unhealthy {
                    reason: format!("no successful HTTP probe in {timeout_secs} s"),
                }
            } else {
                ContainerHealth::Starting
            }
        }
    }
}

/// The aliases, without order or duplicates.
pub(crate) fn alias_set(aliases: &[String]) -> BTreeSet<String> {
    aliases.iter().cloned().collect()
}

fn secs_to_ms(secs: u32) -> u64 {
    u64::from(secs) * 1_000
}
