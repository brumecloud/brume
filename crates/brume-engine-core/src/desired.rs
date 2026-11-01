//! The desired state of the machine, computed from the `World` and the values in storage.

use brume_domain::{
    ContainerPath, Deployment, DeploymentPhase, RevisionValue, Service, ServiceId, World,
};
use brume_proto::agent::{
    ContainerSpec, DesiredState, EnvValue, EnvVar, Healthcheck, IngressRoute, IngressSpec, Mount,
    NetworkAttachment, NetworkSpec, VolumeSpec,
};

use crate::storage::{Storage, StorageError};

/// How long an HTTP healthcheck may take before the container is unhealthy.
pub const HTTP_HEALTHCHECK_TIMEOUT_SECS: u32 = 60;

/// How long a container without an HTTP healthcheck must run to be healthy.
pub const RUNNING_GRACE_SECS: u32 = 5;

/// How long Docker waits for a container with a volume to stop: a database flushes.
const STOP_TIMEOUT_WITH_VOLUME_SECS: u32 = 60;

/// How long Docker waits for any other container to stop.
const STOP_TIMEOUT_SECS: u32 = 10;

/// The last 8 hexadecimal characters of an id. A UUIDv7 starts with its millisecond clock, so
/// its first characters repeat for about a minute; its last ones are random.
pub fn short_id(id: &str) -> &str {
    id.get(id.len().saturating_sub(8)..).unwrap_or(id)
}

/// The internal network of an environment.
pub fn environment_network(environment: &str) -> String {
    format!("brume-env-{}", short_id(environment))
}

/// The network with an internet route of an environment.
pub fn egress_network(environment: &str) -> String {
    format!("brume-egress-{}", short_id(environment))
}

/// The internal network that a service with a public domain shares with Traefik alone.
pub fn edge_network(service: &str) -> String {
    format!("brume-edge-{service}")
}

/// The volume of the `index`-th path that the service's images declared.
pub fn volume_name(service: &ServiceId, index: usize) -> String {
    format!("brume-vol-{}-{index}", short_id(service.as_str()))
}

/// The container of a deployment.
pub fn container_name(service: &Service, deployment: &Deployment) -> String {
    format!(
        "brume-{}-{}",
        service.slug,
        short_id(deployment.id.as_str())
    )
}

/// Everything that must run on the machine at log position `position`.
pub fn desired_state(
    world: &World,
    storage: &impl Storage,
    position: u64,
) -> Result<DesiredState, StorageError> {
    let mut networks = Vec::new();
    for environment in world.environments() {
        let mut services = world.services_in(&environment.id).peekable();
        if services.peek().is_none() {
            continue;
        }
        let egress = services.any(|service| service.egress);
        networks.push(NetworkSpec {
            name: environment_network(environment.id.as_str()),
            internal: true,
            environment_id: environment.id.to_string(),
        });
        if egress {
            networks.push(NetworkSpec {
                name: egress_network(environment.id.as_str()),
                internal: false,
                environment_id: environment.id.to_string(),
            });
        }
    }

    // The volumes of deleted services stay: deleting a service keeps its data.
    let volumes = world
        .services()
        .flat_map(|service| {
            (0..service.volumes.len()).map(|index| VolumeSpec {
                name: volume_name(&service.id, index),
                service_id: service.id.to_string(),
            })
        })
        .collect();

    let mut containers = Vec::new();
    let mut routes = Vec::new();
    for service in world.services().filter(|service| !service.deleted) {
        let active = world.active_deployment(&service.id);
        let in_progress = world.in_progress_deployment(&service.id);
        let started = in_progress.filter(|deployment| {
            matches!(
                deployment.phase,
                DeploymentPhase::Starting | DeploymentPhase::Promoting
            )
        });
        // Two containers never share a volume: when the new revision mounts a volume of the
        // active one, the active container stops first, and comes back if the deployment
        // fails. Only the revisions that run count, never one that failed.
        let stop_active = match (active, started) {
            (Some(active), Some(started)) => {
                let mounted = revision_volumes(world, active);
                revision_volumes(world, started)
                    .iter()
                    .any(|path| mounted.contains(path))
            }
            _ => false,
        };
        // Each running deployment, and whether it holds the service's alias.
        let mut running = Vec::with_capacity(2);
        if let Some(active) = active
            && !stop_active
        {
            running.push((active, true));
        }
        if let Some(deployment) = in_progress {
            match deployment.phase {
                DeploymentPhase::Starting => running.push((deployment, false)),
                DeploymentPhase::Promoting => running.push((deployment, true)),
                _ => {}
            }
        }
        for &(deployment, alias) in &running {
            containers.push(container(world, storage, service, deployment, alias)?);
        }
        // The route follows the alias: the active deployment's domain, else the promoting
        // one's.
        let route = running
            .iter()
            .filter(|(_, alias)| *alias)
            .find_map(|(deployment, _)| public_host(world, deployment));
        if let Some((host, port)) = route {
            let network = edge_network(service.id.as_str());
            networks.push(NetworkSpec {
                name: network.clone(),
                internal: true,
                environment_id: service.environment.to_string(),
            });
            routes.push(IngressRoute {
                host,
                network,
                target: service.slug.to_string(),
                port,
            });
        }
    }

    let ingress = match world.ingress() {
        Some(_) => Some(IngressSpec {
            tunnel_token: storage
                .ingress_token()?
                .ok_or_else(|| StorageError("the tunnel token is missing".to_owned()))?,
            routes,
        }),
        None => None,
    };

    Ok(DesiredState {
        position,
        networks,
        volumes,
        containers,
        ingress,
    })
}

/// The public host and the port of a deployment's revision, when it has a domain and the
/// public entry is set up.
pub(crate) fn public_host(world: &World, deployment: &Deployment) -> Option<(String, u16)> {
    let ingress = world.ingress()?;
    let spec = &world.revision(deployment.revision.as_ref()?)?.spec;
    Some((ingress.host(spec.domain.as_ref()?), spec.port?.get()))
}

/// The volume paths that a deployment's revision mounts; none before it has a revision.
fn revision_volumes<'a>(world: &'a World, deployment: &Deployment) -> &'a [ContainerPath] {
    deployment
        .revision
        .as_ref()
        .and_then(|revision| world.revision(revision))
        .map_or(&[], |revision| revision.spec.volumes.as_slice())
}

/// The name and path of the first volume that the service's containers mount: those of the
/// active revision, else of the deployment in progress once it has one.
pub(crate) fn current_volume(world: &World, service: &Service) -> Option<(String, ContainerPath)> {
    let deployment = world
        .active_deployment(&service.id)
        .or_else(|| world.in_progress_deployment(&service.id))?;
    let path = revision_volumes(world, deployment).first()?;
    let index = service.volumes.iter().position(|known| known == path)?;
    Some((volume_name(&service.id, index), path.clone()))
}

/// The container of a started deployment. `alias`: whether it holds the service's slug on the
/// environment network.
fn container(
    world: &World,
    storage: &impl Storage,
    service: &Service,
    deployment: &Deployment,
    alias: bool,
) -> Result<ContainerSpec, StorageError> {
    let revision = deployment
        .revision
        .as_ref()
        .and_then(|revision| world.revision(revision))
        .ok_or_else(|| StorageError("a started deployment has no revision".to_owned()))?;
    let spec = &revision.spec;

    let mut env = Vec::with_capacity(spec.variables.len());
    for variable in &spec.variables {
        let value = match &variable.value {
            RevisionValue::Plain { blob, .. } => {
                let bytes = storage
                    .blob(blob)?
                    .ok_or_else(|| StorageError(format!("blob {blob} is missing")))?;
                EnvValue::Plain(String::from_utf8_lossy(&bytes).into_owned())
            }
            RevisionValue::Secret { .. } => {
                let resolved = deployment
                    .secrets
                    .iter()
                    .find(|secret| secret.name == variable.name)
                    .ok_or_else(|| {
                        StorageError(format!("secret {} was not resolved", variable.name))
                    })?;
                let secret = storage
                    .secret(&resolved.variable, resolved.version)?
                    .ok_or_else(|| {
                        StorageError(format!(
                            "version {} of secret {} is missing",
                            resolved.version, variable.name
                        ))
                    })?;
                EnvValue::Secret(secret)
            }
        };
        env.push(EnvVar {
            name: variable.name.to_string(),
            value,
        });
    }

    let environment = service.environment.as_str();
    let mut networks = vec![NetworkAttachment {
        network: environment_network(environment),
        aliases: if alias {
            vec![service.slug.to_string()]
        } else {
            Vec::new()
        },
    }];
    if service.egress {
        networks.push(NetworkAttachment {
            network: egress_network(environment),
            aliases: Vec::new(),
        });
    }
    // Traefik reaches the container that holds the alias on the service's edge network.
    if alias && public_host(world, deployment).is_some() {
        networks.push(NetworkAttachment {
            network: edge_network(service.id.as_str()),
            aliases: vec![service.slug.to_string()],
        });
    }

    let mounts = spec
        .volumes
        .iter()
        .map(|path| {
            let index = service
                .volumes
                .iter()
                .position(|known| known == path)
                .expect("a revision's volumes are the service's");
            Mount {
                volume: volume_name(&service.id, index),
                path: path.to_string(),
            }
        })
        .collect::<Vec<_>>();

    let healthcheck = match (spec.port, &spec.healthcheck_path) {
        (Some(port), Some(path)) => Healthcheck::Http {
            port: port.get(),
            path: path.to_string(),
            timeout_secs: HTTP_HEALTHCHECK_TIMEOUT_SECS,
        },
        _ => Healthcheck::Running {
            grace_secs: RUNNING_GRACE_SECS,
        },
    };

    Ok(ContainerSpec {
        name: container_name(service, deployment),
        image: spec.digest.to_string(),
        environment_id: service.environment.to_string(),
        service_id: service.id.to_string(),
        deployment_id: deployment.id.to_string(),
        env,
        command: spec.start_command.as_ref().map(|command| command.argv()),
        networks,
        stop_timeout_secs: if mounts.is_empty() {
            STOP_TIMEOUT_SECS
        } else {
            STOP_TIMEOUT_WITH_VOLUME_SECS
        },
        mounts,
        cpu_millis: spec.cpu.get(),
        memory_mb: spec.memory.get(),
        healthcheck,
    })
}
