//! The agent's door to Docker: snapshots, and each `Action` as a bollard call. The env strings
//! of containers, with the values of their secrets, are built here and nowhere else.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use bollard::errors::Error;
use bollard::models::{
    ContainerCreateBody, ContainerInspectResponse, ContainerStateStatusEnum,
    ContainerSummaryStateEnum, EndpointIpamConfig, EndpointSettings, HostConfig,
    HostConfigLogConfig, Ipam, IpamConfig, Mount, MountPoint, MountType, Network,
    NetworkConnectRequest, NetworkCreateRequest, NetworkDisconnectRequest, NetworkingConfig,
    RestartPolicyNameEnum, VolumeCreateRequest,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, InspectContainerOptions, ListContainersOptionsBuilder,
    ListImagesOptions, ListNetworksOptions, ListVolumesOptions, RemoveContainerOptionsBuilder,
    StartContainerOptions, StopContainerOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use brume_agent_core::config::{
    ContainerConfig, HostDir, LABEL_COMPONENT, LABEL_CONFIG_HASH, LABEL_DEPLOYMENT,
    LABEL_ENVIRONMENT, LABEL_MANAGED, LABEL_SERVICE, RestartPolicy,
};
use brume_agent_core::observed::{
    DockerSnapshot, SnapContainer, SnapNetwork, SnapState, SnapVolume,
};
use brume_agent_core::plan::Action;
use brume_proto::agent::{EnvValue, NetworkAttachment, NetworkSpec};
use secrecy::ExposeSecret;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::jobs::pull;
use crate::probe::{LABEL_PROBE, probe};

/// How long bollard waits for the headers of a response. A stream (logs, pulls) is not cut.
const REQUEST_TIMEOUT_SECS: u64 = 120;

/// Where a managed container's log lines come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogSource {
    pub environment_id: String,
    pub service_id: String,
    pub deployment_id: String,
    pub running: bool,
}

/// A Docker snapshot, and the log source of each managed app container, by name. The
/// components' logs stay in Docker.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observation {
    pub snapshot: DockerSnapshot,
    pub sources: BTreeMap<String, LogSource>,
    /// The managed containers left out of the snapshot because Docker failed to inspect them,
    /// with its error.
    pub unreadable: Vec<(String, String)>,
}

/// What an action gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    /// The result of a `Probe`.
    Probed {
        success: bool,
    },
}

/// Connects to the local Docker: bollard's local defaults, else `~/.docker/run/docker.sock`,
/// with the API version negotiated, then a ping.
pub async fn connect() -> Result<Docker, Error> {
    let first = match ready(Docker::connect_with_local_defaults()).await {
        Ok(docker) => return Ok(docker),
        Err(error) => error,
    };
    let Some(home) = std::env::var_os("HOME") else {
        return Err(first);
    };
    let socket = PathBuf::from(home).join(".docker/run/docker.sock");
    let Some(socket) = socket.to_str() else {
        return Err(first);
    };
    ready(Docker::connect_with_unix(
        socket,
        REQUEST_TIMEOUT_SECS,
        API_DEFAULT_VERSION,
    ))
    .await
    .map_err(|_| first)
}

/// `docker` with the API version that the daemon and bollard share, once it answers a ping.
/// Docker 29 refuses the versions under 1.44.
async fn ready(docker: Result<Docker, Error>) -> Result<Docker, Error> {
    let docker = docker?.negotiate_version().await?;
    docker.ping().await?;
    Ok(docker)
}

/// Docker, as the agent drives it.
#[derive(Clone, Debug)]
pub struct DockerShell {
    docker: Docker,
    /// A label added to everything the shell creates, which also limits the containers it
    /// sees to those that carry it. Tests use it to stay away from real containers.
    scope: Option<(String, String)>,
    /// `HostDir::TraefikConfig`: where the agent writes Traefik's config.
    traefik_dir: Option<PathBuf>,
}

impl DockerShell {
    pub fn new(docker: Docker) -> Self {
        Self {
            docker,
            scope: None,
            traefik_dir: None,
        }
    }

    /// Sets the directory of Traefik's config, which Traefik mounts. Without it, Traefik is
    /// never created.
    pub fn with_traefik_dir(mut self, dir: PathBuf) -> Self {
        self.traefik_dir = Some(dir);
        self
    }

    /// The directory of Traefik's config, if set.
    pub fn traefik_dir(&self) -> Option<&Path> {
        self.traefik_dir.as_deref()
    }

    /// Labels everything the shell creates with `key=value`, and limits the containers it sees
    /// to those labelled so.
    pub fn scoped(mut self, key: &str, value: &str) -> Self {
        self.scope = Some((key.to_owned(), value.to_owned()));
        self
    }

    pub fn docker(&self) -> &Docker {
        &self.docker
    }

    /// The labels to add to what the shell creates.
    pub fn extra_labels(&self) -> BTreeMap<String, String> {
        self.scope.iter().cloned().collect()
    }

    /// Lists networks, volumes, images and containers. A managed container is inspected for
    /// its start, exit code, networks, volumes and labels; one that Docker fails to inspect is
    /// left out, in `unreadable`, and the others go on. Any other container keeps its state and
    /// volumes from the listing, so that a Brume container never shares a volume with it.
    pub async fn observe(&self) -> Result<Observation, Error> {
        let networks = self
            .docker
            .list_networks(None::<ListNetworksOptions>)
            .await?
            .into_iter()
            .filter_map(snap_network)
            .collect();
        let volumes = self
            .docker
            .list_volumes(None::<ListVolumesOptions>)
            .await?
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|volume| SnapVolume {
                managed: is_managed(Some(&volume.labels)),
                name: volume.name,
            })
            .collect();
        let images = self
            .docker
            .list_images(None::<ListImagesOptions>)
            .await?
            .into_iter()
            .flat_map(|image| image.repo_digests)
            .collect();

        let mut list = ListContainersOptionsBuilder::new().all(true);
        if let Some((key, value)) = &self.scope {
            let filter = HashMap::from([("label", vec![format!("{key}={value}")])]);
            list = list.filters(&filter);
        }
        let mut observation = Observation::default();
        for summary in self.docker.list_containers(Some(list.build())).await? {
            let Some(name) = summary
                .names
                .as_ref()
                .and_then(|names| names.first())
                .map(|name| name.trim_start_matches('/').to_owned())
            else {
                continue;
            };
            if !is_managed(summary.labels.as_ref()) {
                observation.snapshot.containers.push(unmanaged(
                    name,
                    summary.state,
                    summary.mounts,
                ));
                continue;
            }
            let inspected = self
                .docker
                .inspect_container(&name, None::<InspectContainerOptions>)
                .await;
            let Some(inspect) = readable(&name, inspected, summary.mounts, &mut observation) else {
                continue;
            };
            let (container, source) = managed(name, inspect);
            if let Some(source) = source {
                observation.sources.insert(container.name.clone(), source);
            }
            observation.snapshot.containers.push(container);
        }
        observation.snapshot.networks = networks;
        observation.snapshot.volumes = volumes;
        observation.snapshot.images = images;
        Ok(observation)
    }

    /// Runs `action`.
    pub async fn execute(&self, action: &Action) -> Result<Outcome, Error> {
        match action {
            Action::CreateNetwork(spec) => {
                let request = network_request(spec, self.labels());
                ignore_status(self.docker.create_network(request).await, CONFLICT)?;
            }
            Action::RemoveNetwork(name) => {
                ignore_status(self.docker.remove_network(name).await, NOT_FOUND)?;
            }
            Action::CreateVolume(spec) => {
                let mut labels = self.labels();
                labels.insert(LABEL_SERVICE.to_owned(), spec.service_id.clone());
                let request = VolumeCreateRequest {
                    name: Some(spec.name.clone()),
                    driver: Some("local".to_owned()),
                    labels: Some(labels),
                    ..VolumeCreateRequest::default()
                };
                self.docker.create_volume(request).await?;
            }
            Action::PullImage(image) => pull(&self.docker, image).await?,
            Action::CreateContainer(config) => {
                if config.skipped_env > 0 {
                    // Never their names: a malformed name may hold a value.
                    tracing::warn!(
                        container = %config.name,
                        skipped = config.skipped_env,
                        "variables whose name is not [A-Za-z_][A-Za-z0-9_]* are left out"
                    );
                }
                let options = CreateContainerOptionsBuilder::new()
                    .name(&config.name)
                    .build();
                let body = create_body(config, &self.extra_labels(), self.traefik_dir())?;
                self.docker.create_container(Some(options), body).await?;
            }
            Action::StartContainer(name) => {
                self.docker
                    .start_container(name, None::<StartContainerOptions>)
                    .await?;
            }
            Action::StopContainer { name, timeout_secs } => {
                let timeout = i32::try_from(*timeout_secs).unwrap_or(i32::MAX);
                let options = StopContainerOptionsBuilder::new().t(timeout).build();
                ignore_status(
                    self.docker.stop_container(name, Some(options)).await,
                    NOT_FOUND,
                )?;
            }
            Action::RemoveContainer(name) => {
                // `v` removes the anonymous volumes only: named volumes stay.
                let options = RemoveContainerOptionsBuilder::new()
                    .force(true)
                    .v(true)
                    .build();
                ignore_status(
                    self.docker.remove_container(name, Some(options)).await,
                    NOT_FOUND,
                )?;
            }
            Action::Reattach {
                container,
                network,
                aliases,
                ipv4_address,
            } => {
                // The container keeps its address: Docker would hand it the lowest free one,
                // and the connections it opened while it started, such as an API's pool to its
                // database, would hang without an error.
                let address = match ipv4_address {
                    Some(address) => Some(address.clone()),
                    None => self.address_on(container, network).await,
                };
                let disconnect = NetworkDisconnectRequest {
                    container: container.clone(),
                    force: Some(true),
                };
                // It fails when the container is not attached, which is fine.
                let _ = self.docker.disconnect_network(network, disconnect).await;
                let connect = |address: Option<&str>| NetworkConnectRequest {
                    container: container.clone(),
                    endpoint_config: Some(endpoint_settings(aliases, address)),
                };
                let connected = self
                    .docker
                    .connect_network(network, connect(address.as_deref()))
                    .await;
                match connected {
                    Ok(()) => {}
                    // Another container took the address in between: any address beats none.
                    Err(error) if ipv4_address.is_none() && address.is_some() => {
                        tracing::warn!(%error, container, network, "the container changes address");
                        self.docker.connect_network(network, connect(None)).await?;
                    }
                    Err(error) => return Err(error),
                }
            }
            Action::Disconnect { container, network } => {
                let disconnect = NetworkDisconnectRequest {
                    container: container.clone(),
                    force: Some(true),
                };
                ignore_status(
                    self.docker.disconnect_network(network, disconnect).await,
                    NOT_FOUND,
                )?;
            }
            Action::Probe {
                container,
                network,
                port,
                path,
            } => {
                let success = probe(
                    &self.docker,
                    &self.extra_labels(),
                    network,
                    container,
                    *port,
                    path,
                )
                .await;
                return Ok(Outcome::Probed { success });
            }
        }
        Ok(Outcome::Done)
    }

    /// Removes the probe containers that a crash of the agent left behind. A removal that
    /// fails is logged, and the others go on.
    pub async fn sweep_probes(&self) -> Result<(), Error> {
        let mut labels = vec![format!("{LABEL_PROBE}=1")];
        if let Some((key, value)) = &self.scope {
            labels.push(format!("{key}={value}"));
        }
        let filters = HashMap::from([("label", labels)]);
        let list = ListContainersOptionsBuilder::new()
            .all(true)
            .filters(&filters)
            .build();
        for container in self.docker.list_containers(Some(list)).await? {
            let Some(id) = container.id else {
                continue;
            };
            let options = RemoveContainerOptionsBuilder::new().force(true).build();
            let removed = self.docker.remove_container(&id, Some(options)).await;
            if let Err(error) = ignore_status(removed, NOT_FOUND) {
                tracing::warn!(%error, id, "cannot remove a leftover probe container");
            }
        }
        Ok(())
    }

    /// `brume.managed=1` and the scope label, for a network or a volume.
    fn labels(&self) -> HashMap<String, String> {
        let mut labels: HashMap<String, String> = self.extra_labels().into_iter().collect();
        labels.insert(LABEL_MANAGED.to_owned(), "1".to_owned());
        labels
    }

    /// The address of `container` on `network`; `None` if it is not attached, stopped, or
    /// Docker fails to inspect it.
    async fn address_on(&self, container: &str, network: &str) -> Option<String> {
        let inspect = self
            .docker
            .inspect_container(container, None::<InspectContainerOptions>)
            .await
            .ok()?;
        address_on_network(inspect, network)
    }
}

/// A network as Docker lists it, with the subnets of its IPAM config; `None` without a name.
fn snap_network(network: Network) -> Option<SnapNetwork> {
    let subnets = network
        .ipam
        .and_then(|ipam| ipam.config)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|config| config.subnet)
        .collect();
    Some(SnapNetwork {
        managed: is_managed(network.labels.as_ref()),
        name: network.name?,
        internal: network.internal.unwrap_or(false),
        subnets,
    })
}

/// The body of `POST /networks/create` for `spec`, with `labels` and the environment's. A
/// subnet, when the spec has one, is the IPAM config's only one.
fn network_request(
    spec: &NetworkSpec,
    mut labels: HashMap<String, String>,
) -> NetworkCreateRequest {
    if !spec.environment_id.is_empty() {
        labels.insert(LABEL_ENVIRONMENT.to_owned(), spec.environment_id.clone());
    }
    let ipam = spec.subnet.as_ref().map(|subnet| Ipam {
        config: Some(vec![IpamConfig {
            subnet: Some(subnet.clone()),
            ..IpamConfig::default()
        }]),
        ..Ipam::default()
    });
    NetworkCreateRequest {
        name: spec.name.clone(),
        driver: Some("bridge".to_owned()),
        internal: Some(spec.internal),
        labels: Some(labels),
        ipam,
        ..NetworkCreateRequest::default()
    }
}

/// The settings of a container on a network: its aliases, and its fixed address if any.
/// The IPv4 address of the inspected container on `network`, if it has one there.
fn address_on_network(inspect: ContainerInspectResponse, network: &str) -> Option<String> {
    inspect
        .network_settings?
        .networks?
        .remove(network)?
        .ip_address
        .filter(|address| !address.is_empty())
}

fn endpoint_settings(aliases: &[String], ipv4_address: Option<&str>) -> EndpointSettings {
    EndpointSettings {
        aliases: Some(aliases.to_vec()),
        ipam_config: ipv4_address.map(|address| EndpointIpamConfig {
            ipv4_address: Some(address.to_owned()),
            ..EndpointIpamConfig::default()
        }),
        ..EndpointSettings::default()
    }
}

/// Whether `labels` has `brume.managed=1`.
fn is_managed(labels: Option<&HashMap<String, String>>) -> bool {
    labels
        .and_then(|labels| labels.get(LABEL_MANAGED))
        .is_some_and(|value| value == "1")
}

/// The inspection of the container `name`, or `None` to leave it out of the snapshot: removed
/// since the listing, or unreadable, which `observation` keeps for the logs. An unreadable
/// container still holds the volumes that the listing gave, `listed_mounts`.
fn readable(
    name: &str,
    inspected: Result<ContainerInspectResponse, Error>,
    listed_mounts: Option<Vec<MountPoint>>,
    observation: &mut Observation,
) -> Option<ContainerInspectResponse> {
    match inspected {
        Ok(inspect) => Some(inspect),
        Err(Error::DockerResponseServerError {
            status_code: NOT_FOUND,
            ..
        }) => None,
        Err(error) => {
            observation
                .unreadable
                .push((name.to_owned(), error.to_string()));
            observation
                .snapshot
                .uninspected_mounts
                .extend(volume_names(listed_mounts));
            None
        }
    }
}

const CONFLICT: u16 = 409;
const NOT_FOUND: u16 = 404;

/// `result`, with an error of HTTP status `status` taken as success.
fn ignore_status<T>(result: Result<T, Error>, status: u16) -> Result<(), Error> {
    match result {
        Ok(_) => Ok(()),
        Err(Error::DockerResponseServerError { status_code, .. }) if status_code == status => {
            Ok(())
        }
        Err(error) => Err(error),
    }
}

/// A container without the `brume.managed` label: its name, its state and its volumes, nothing
/// else the agent uses.
fn unmanaged(
    name: String,
    state: Option<ContainerSummaryStateEnum>,
    mounts: Option<Vec<MountPoint>>,
) -> SnapContainer {
    let state = match state {
        Some(ContainerSummaryStateEnum::CREATED) => SnapState::Created,
        Some(ContainerSummaryStateEnum::RUNNING) => SnapState::Running,
        Some(ContainerSummaryStateEnum::EXITED) => SnapState::Exited { code: 0 },
        _ => SnapState::Other,
    };
    SnapContainer {
        name,
        managed: false,
        deployment_id: String::new(),
        image: String::new(),
        state,
        started_at_ms: None,
        networks: Vec::new(),
        mounts: volume_names(mounts),
        config_hash: String::new(),
        stop_timeout_secs: 0,
    }
}

/// The named volumes among `mounts`: neither bind mounts nor tmpfs.
fn volume_names(mounts: Option<Vec<MountPoint>>) -> Vec<String> {
    mounts
        .unwrap_or_default()
        .into_iter()
        .filter(|mount| mount.typ.as_deref() == Some("volume"))
        .filter_map(|mount| mount.name)
        .collect()
}

/// A managed container from its inspection, and where its logs come from: nowhere for a
/// component, whose logs stay in Docker.
fn managed(name: String, inspect: ContainerInspectResponse) -> (SnapContainer, Option<LogSource>) {
    let state = inspect.state.unwrap_or_default();
    let snap_state = match state.status {
        Some(ContainerStateStatusEnum::CREATED) => SnapState::Created,
        Some(ContainerStateStatusEnum::RUNNING) => SnapState::Running,
        Some(ContainerStateStatusEnum::EXITED) => SnapState::Exited {
            code: state.exit_code.unwrap_or(0),
        },
        _ => SnapState::Other,
    };
    let started_at_ms = state.started_at.as_deref().and_then(epoch_ms);
    let config = inspect.config.unwrap_or_default();
    let labels = config.labels.unwrap_or_default();
    let label = |key: &str| labels.get(key).cloned().unwrap_or_default();
    let short_id: String = inspect.id.unwrap_or_default().chars().take(12).collect();
    let mut networks: Vec<NetworkAttachment> = inspect
        .network_settings
        .and_then(|settings| settings.networks)
        .unwrap_or_default()
        .into_iter()
        .map(|(network, endpoint)| {
            // Older APIs add the name and short id; they are not aliases that Brume sets.
            let aliases = endpoint
                .aliases
                .unwrap_or_default()
                .into_iter()
                .filter(|alias| *alias != name && *alias != short_id)
                .collect();
            NetworkAttachment { network, aliases }
        })
        .collect();
    networks.sort_by(|left, right| left.network.cmp(&right.network));
    let source = (!labels.contains_key(LABEL_COMPONENT)).then(|| LogSource {
        environment_id: label(LABEL_ENVIRONMENT),
        service_id: label(LABEL_SERVICE),
        deployment_id: label(LABEL_DEPLOYMENT),
        running: snap_state == SnapState::Running,
    });
    let container = SnapContainer {
        name,
        managed: true,
        deployment_id: label(LABEL_DEPLOYMENT),
        image: config.image.unwrap_or_default(),
        state: snap_state,
        started_at_ms,
        networks,
        mounts: volume_names(inspect.mounts),
        config_hash: label(LABEL_CONFIG_HASH),
        stop_timeout_secs: config
            .stop_timeout
            .and_then(|secs| u32::try_from(secs).ok())
            .unwrap_or(DOCKER_STOP_TIMEOUT_SECS),
    };
    (container, source)
}

/// Docker's stop timeout when the container sets none.
const DOCKER_STOP_TIMEOUT_SECS: u32 = 10;

/// An RFC 3339 instant in milliseconds since the Unix epoch; `None` for Docker's zero time
/// (`0001-01-01T00:00:00Z`, never started) or an unreadable one.
fn epoch_ms(instant: &str) -> Option<u64> {
    let nanos = OffsetDateTime::parse(instant, &Rfc3339)
        .ok()?
        .unix_timestamp_nanos();
    u64::try_from(nanos / 1_000_000).ok().filter(|ms| *ms > 0)
}

/// The body of `POST /containers/create` for `config`, with `extra_labels`. The only place
/// where a secret leaves its `SecretString`. A bind mount of `HostDir::TraefikConfig` takes
/// `traefik_dir`, read-only; without it, the config cannot be created.
pub(crate) fn create_body(
    config: &ContainerConfig,
    extra_labels: &BTreeMap<String, String>,
    traefik_dir: Option<&Path>,
) -> Result<ContainerCreateBody, Error> {
    let env = config
        .env
        .iter()
        .map(|var| match &var.value {
            EnvValue::Plain(value) => format!("{}={value}", var.name),
            EnvValue::Secret(secret) => format!("{}={}", var.name, secret.expose_secret()),
        })
        .collect();
    let mut labels: HashMap<String, String> = extra_labels.clone().into_iter().collect();
    labels.extend(config.labels.clone());
    let host = &config.host_config;
    let restart_policy = match host.restart_policy {
        RestartPolicy::No => RestartPolicyNameEnum::NO,
    };
    let mut mounts: Vec<Mount> = host
        .mounts
        .iter()
        .map(|mount| Mount {
            typ: Some(MountType::VOLUME),
            source: Some(mount.source.clone()),
            target: Some(mount.target.clone()),
            ..Mount::default()
        })
        .collect();
    for bind in &host.binds {
        let source = match bind.source {
            HostDir::TraefikConfig => traefik_dir.ok_or_else(|| Error::IOError {
                err: std::io::Error::other("the agent has no directory for Traefik's config"),
            })?,
        };
        mounts.push(Mount {
            typ: Some(MountType::BIND),
            source: Some(source.to_string_lossy().into_owned()),
            target: Some(bind.target.clone()),
            read_only: Some(true),
            ..Mount::default()
        });
    }
    let host_config = HostConfig {
        network_mode: Some(host.network_mode.clone()),
        cap_drop: Some(host.cap_drop.clone()),
        cap_add: Some(host.cap_add.clone()),
        security_opt: Some(host.security_opt.clone()),
        memory: Some(host.memory_bytes),
        memory_swap: Some(host.memory_swap_bytes),
        nano_cpus: Some(host.nano_cpus),
        pids_limit: Some(host.pids_limit),
        restart_policy: Some(bollard::models::RestartPolicy {
            name: Some(restart_policy),
            maximum_retry_count: None,
        }),
        log_config: Some(HostConfigLogConfig {
            typ: Some(host.log_config.driver.clone()),
            config: Some(host.log_config.options.clone().into_iter().collect()),
        }),
        mounts: Some(mounts),
        privileged: Some(host.privileged),
        publish_all_ports: Some(host.publish_all_ports),
        readonly_rootfs: Some(host.readonly_rootfs),
        ..HostConfig::default()
    };
    let endpoints = config
        .endpoints
        .iter()
        .map(|endpoint| {
            let settings = endpoint_settings(&endpoint.aliases, endpoint.ipv4_address.as_deref());
            (endpoint.network.clone(), settings)
        })
        .collect();
    Ok(ContainerCreateBody {
        image: Some(config.image.clone()),
        env: Some(env),
        cmd: config.cmd.clone(),
        user: config.user.clone(),
        labels: Some(labels),
        stop_timeout: Some(i64::from(config.stop_timeout_secs)),
        host_config: Some(host_config),
        networking_config: Some(NetworkingConfig {
            endpoints_config: Some(endpoints),
        }),
        ..ContainerCreateBody::default()
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use bollard::errors::Error;
    use bollard::models::{
        ContainerConfig as DockerConfig, ContainerInspectResponse, ContainerState,
        ContainerStateStatusEnum, ContainerSummaryStateEnum, EndpointSettings, HostConfig,
        HostConfigLogConfig, Mount, MountPoint, MountType, NetworkSettings, RestartPolicy,
        RestartPolicyNameEnum,
    };
    use std::path::Path;

    use bollard::models::{EndpointIpamConfig, Ipam, IpamConfig, Network};
    use brume_agent_core::config::{ContainerConfig, container_config};
    use brume_agent_core::ingress::{CLOUDFLARED_CONTAINER, TRAEFIK_CONTAINER, ingress_components};
    use brume_agent_core::observed::SnapState;
    use brume_proto::agent::{
        ContainerSpec, EnvValue, EnvVar, Healthcheck, IngressRoute, IngressSpec,
        Mount as SpecMount, NetworkAttachment, NetworkSpec,
    };

    use super::{
        Observation, address_on_network, create_body, epoch_ms, managed, network_request, readable,
        snap_network, unmanaged,
    };

    /// A named volume, a bind mount of the host and a tmpfs, as Docker lists them.
    fn mount_points() -> Vec<MountPoint> {
        let point = |typ: &str, name: Option<&str>| MountPoint {
            typ: Some(typ.to_owned()),
            name: name.map(ToOwned::to_owned),
            destination: Some("/data".to_owned()),
            ..MountPoint::default()
        };
        vec![
            point("volume", Some("brume-vol-a")),
            point("bind", None),
            point("tmpfs", None),
        ]
    }

    #[test]
    fn unmanaged_container_reports_its_volumes() {
        let container = unmanaged(
            "someone-else".to_owned(),
            Some(ContainerSummaryStateEnum::RUNNING),
            Some(mount_points()),
        );
        assert!(!container.managed);
        assert_eq!(container.state, SnapState::Running);
        assert_eq!(container.mounts, ["brume-vol-a"]);
    }

    #[test]
    fn an_unreadable_container_is_left_out_but_keeps_its_volumes() {
        let mut observation = Observation::default();
        let removed = Err(Error::DockerResponseServerError {
            status_code: 404,
            message: "No such container".to_owned(),
        });
        assert!(
            readable(
                "brume-gone",
                removed,
                Some(mount_points()),
                &mut observation
            )
            .is_none()
        );
        assert!(observation.unreadable.is_empty());
        // Gone: it holds nothing.
        assert!(observation.snapshot.uninspected_mounts.is_empty());

        let broken = Err(Error::DockerResponseServerError {
            status_code: 500,
            message: "boom".to_owned(),
        });
        assert!(readable("brume-web", broken, Some(mount_points()), &mut observation).is_none());
        assert_eq!(observation.unreadable.len(), 1);
        assert_eq!(observation.unreadable[0].0, "brume-web");
        assert!(observation.unreadable[0].1.contains("500"));
        // Its named volume, from the listing, still blocks a create on it.
        assert_eq!(observation.snapshot.uninspected_mounts, ["brume-vol-a"]);

        let fine = Ok(ContainerInspectResponse::default());
        assert!(readable("brume-db", fine, Some(mount_points()), &mut observation).is_some());
        assert_eq!(observation.unreadable.len(), 1);
        assert_eq!(observation.snapshot.uninspected_mounts, ["brume-vol-a"]);
    }

    #[test]
    fn docker_zero_time_is_never_started() {
        assert_eq!(epoch_ms("0001-01-01T00:00:00Z"), None);
        assert_eq!(epoch_ms("garbage"), None);
        assert_eq!(
            epoch_ms("2026-10-31T12:34:56.123456789Z"),
            Some(1_793_450_096_123)
        );
    }

    #[test]
    fn inspected_container_keeps_brume_aliases_only() {
        let labels = HashMap::from([
            ("brume.managed".to_owned(), "1".to_owned()),
            ("brume.environment".to_owned(), "env".to_owned()),
            ("brume.service".to_owned(), "svc".to_owned()),
            ("brume.deployment".to_owned(), "dep".to_owned()),
            ("brume.config-hash".to_owned(), "abc".to_owned()),
        ]);
        let endpoint = |aliases: &[&str]| EndpointSettings {
            aliases: Some(strings(aliases)),
            ..EndpointSettings::default()
        };
        let inspect = ContainerInspectResponse {
            id: Some("0123456789abcdef0123".to_owned()),
            state: Some(ContainerState {
                status: Some(ContainerStateStatusEnum::EXITED),
                exit_code: Some(137),
                started_at: Some("2026-10-31T12:34:56Z".to_owned()),
                ..ContainerState::default()
            }),
            config: Some(DockerConfig {
                image: Some("nginx@sha256:aa".to_owned()),
                labels: Some(labels),
                stop_timeout: Some(30),
                ..DockerConfig::default()
            }),
            network_settings: Some(NetworkSettings {
                networks: Some(HashMap::from([
                    ("net-b".to_owned(), endpoint(&[])),
                    (
                        "net-a".to_owned(),
                        endpoint(&["web", "brume-web", "0123456789ab"]),
                    ),
                ])),
                ..NetworkSettings::default()
            }),
            mounts: Some(mount_points()),
            ..ContainerInspectResponse::default()
        };
        let (container, source) = managed("brume-web".to_owned(), inspect);
        let source = source.expect("an app's logs are forwarded");
        assert!(container.managed);
        assert_eq!(container.mounts, ["brume-vol-a"]);
        assert_eq!(container.state, SnapState::Exited { code: 137 });
        assert_eq!(container.started_at_ms, Some(1_793_450_096_000));
        assert_eq!(container.config_hash, "abc");
        assert_eq!(container.deployment_id, "dep");
        assert_eq!(container.stop_timeout_secs, 30);
        assert_eq!(
            container.networks,
            [
                NetworkAttachment {
                    network: "net-a".to_owned(),
                    aliases: strings(&["web"]),
                },
                NetworkAttachment {
                    network: "net-b".to_owned(),
                    aliases: Vec::new(),
                },
            ]
        );
        assert_eq!(source.environment_id, "env");
        assert_eq!(source.service_id, "svc");
        assert!(!source.running);
    }

    #[test]
    fn a_reattach_reads_the_address_on_its_network() {
        let endpoint = |address: &str| EndpointSettings {
            ip_address: Some(address.to_owned()),
            ..EndpointSettings::default()
        };
        let inspect = ContainerInspectResponse {
            network_settings: Some(NetworkSettings {
                networks: Some(HashMap::from([
                    ("brume-env-a".to_owned(), endpoint("10.213.0.4")),
                    ("brume-egress-a".to_owned(), endpoint("10.213.1.2")),
                    // A stopped container keeps its endpoint, without an address.
                    ("brume-env-b".to_owned(), endpoint("")),
                ])),
                ..NetworkSettings::default()
            }),
            ..ContainerInspectResponse::default()
        };
        assert_eq!(
            address_on_network(inspect.clone(), "brume-env-a").as_deref(),
            Some("10.213.0.4")
        );
        assert_eq!(address_on_network(inspect.clone(), "brume-env-b"), None);
        assert_eq!(address_on_network(inspect, "brume-env-c"), None);
        assert_eq!(
            address_on_network(ContainerInspectResponse::default(), "brume-env-a"),
            None
        );
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn create_body_carries_the_config_and_nothing_else() {
        let spec = ContainerSpec {
            name: "brume-web".to_owned(),
            image: "nginx@sha256:aa".to_owned(),
            environment_id: "env".to_owned(),
            service_id: "svc".to_owned(),
            deployment_id: "dep".to_owned(),
            env: vec![
                EnvVar {
                    name: "GREETING".to_owned(),
                    value: EnvValue::Plain("a=b c".to_owned()),
                },
                EnvVar {
                    name: "TOKEN".to_owned(),
                    value: EnvValue::Secret("s3cr3t".into()),
                },
            ],
            command: Some(strings(&["nginx", "-g", "daemon off;"])),
            networks: vec![
                NetworkAttachment {
                    network: "brume-env".to_owned(),
                    aliases: strings(&["web"]),
                },
                NetworkAttachment {
                    network: "brume-egress".to_owned(),
                    aliases: Vec::new(),
                },
            ],
            mounts: vec![SpecMount {
                volume: "brume-vol".to_owned(),
                path: "/data".to_owned(),
            }],
            cpu_millis: 250,
            memory_mb: 64,
            stop_timeout_secs: 20,
            healthcheck: Healthcheck::Running { grace_secs: 1 },
        };
        let config = container_config(&spec);
        let extra = BTreeMap::from([("brume.test".to_owned(), "1".to_owned())]);
        let body = create_body(&config, &extra, None).expect("a body");

        assert_eq!(body.image.as_deref(), Some("nginx@sha256:aa"));
        assert_eq!(body.env, Some(strings(&["GREETING=a=b c", "TOKEN=s3cr3t"])));
        assert_eq!(body.cmd, Some(strings(&["nginx", "-g", "daemon off;"])));
        assert_eq!(body.stop_timeout, Some(20));
        let mut labels: HashMap<String, String> = config.labels.clone().into_iter().collect();
        labels.insert("brume.test".to_owned(), "1".to_owned());
        assert_eq!(body.labels, Some(labels));
        assert_eq!(
            body.host_config,
            Some(HostConfig {
                network_mode: Some("brume-env".to_owned()),
                cap_drop: Some(strings(&["ALL"])),
                cap_add: Some(config.host_config.cap_add.clone()),
                security_opt: Some(strings(&["no-new-privileges:true"])),
                memory: Some(64 * 1024 * 1024),
                memory_swap: Some(64 * 1024 * 1024),
                nano_cpus: Some(250_000_000),
                pids_limit: Some(512),
                restart_policy: Some(RestartPolicy {
                    name: Some(RestartPolicyNameEnum::NO),
                    maximum_retry_count: None,
                }),
                log_config: Some(HostConfigLogConfig {
                    typ: Some("json-file".to_owned()),
                    config: Some(HashMap::from([
                        ("max-file".to_owned(), "3".to_owned()),
                        ("max-size".to_owned(), "10m".to_owned()),
                    ])),
                }),
                mounts: Some(vec![Mount {
                    typ: Some(MountType::VOLUME),
                    source: Some("brume-vol".to_owned()),
                    target: Some("/data".to_owned()),
                    ..Mount::default()
                }]),
                privileged: Some(false),
                publish_all_ports: Some(false),
                readonly_rootfs: Some(false),
                ..HostConfig::default()
            })
        );
        let endpoints = body
            .networking_config
            .and_then(|networking| networking.endpoints_config)
            .expect("endpoints");
        assert_eq!(
            endpoints,
            HashMap::from([
                (
                    "brume-env".to_owned(),
                    EndpointSettings {
                        aliases: Some(strings(&["web"])),
                        ..EndpointSettings::default()
                    }
                ),
                (
                    "brume-egress".to_owned(),
                    EndpointSettings {
                        aliases: Some(Vec::new()),
                        ..EndpointSettings::default()
                    }
                ),
            ])
        );
        // Nothing else is set.
        let empty = bollard::models::ContainerCreateBody {
            image: body.image.clone(),
            env: body.env.clone(),
            cmd: body.cmd.clone(),
            stop_timeout: body.stop_timeout,
            labels: body.labels.clone(),
            host_config: body.host_config.clone(),
            ..Default::default()
        };
        assert_eq!(
            bollard::models::ContainerCreateBody {
                networking_config: None,
                ..create_body(&config, &extra, None).expect("a body")
            },
            empty
        );
    }

    fn ingress() -> IngressSpec {
        IngressSpec {
            tunnel_token: "eyJ-s3cr3t".into(),
            routes: vec![IngressRoute {
                host: "demo.example.com".to_owned(),
                network: "brume-edge-svc".to_owned(),
                target: "brume-edge-svc".to_owned(),
                port: 8080,
            }],
        }
    }

    fn component(name: &str) -> ContainerConfig {
        let (_, containers) = ingress_components(&ingress());
        containers
            .into_iter()
            .find(|config| config.name == name)
            .expect("the component")
    }

    #[test]
    fn traefik_mounts_its_config_read_only_and_listens_on_its_address() {
        let config = component(TRAEFIK_CONTAINER);
        let dir = Path::new("/var/lib/brume/agent/traefik");
        let body = create_body(&config, &BTreeMap::new(), Some(dir)).expect("a body");
        assert_eq!(body.user.as_deref(), Some("65534:65534"));
        let host = body.host_config.expect("a host config");
        assert_eq!(host.readonly_rootfs, Some(true));
        assert_eq!(host.cap_drop, Some(strings(&["ALL"])));
        assert_eq!(host.cap_add, Some(Vec::new()));
        assert_eq!(host.network_mode.as_deref(), Some("brume-ingress"));
        assert_eq!(
            host.mounts,
            Some(vec![Mount {
                typ: Some(MountType::BIND),
                source: Some("/var/lib/brume/agent/traefik".to_owned()),
                target: Some("/etc/traefik/dynamic".to_owned()),
                read_only: Some(true),
                ..Mount::default()
            }])
        );
        assert_eq!(
            host.binds, None,
            "no -v style bind, which creates a missing source"
        );
        let endpoints = body
            .networking_config
            .and_then(|networking| networking.endpoints_config)
            .expect("endpoints");
        assert_eq!(
            endpoints["brume-ingress"].ipam_config,
            Some(EndpointIpamConfig {
                ipv4_address: Some("10.213.255.10".to_owned()),
                ..EndpointIpamConfig::default()
            })
        );
        assert_eq!(endpoints["brume-edge-svc"].ipam_config, None);

        // Without the directory, Traefik cannot be created.
        assert!(create_body(&config, &BTreeMap::new(), None).is_err());
    }

    #[test]
    fn cloudflared_gets_its_token_at_the_docker_call_only() {
        let config = component(CLOUDFLARED_CONTAINER);
        assert!(!format!("{config:?}").contains("s3cr3t"));
        let body = create_body(&config, &BTreeMap::new(), None).expect("a body");
        assert_eq!(body.env, Some(strings(&["TUNNEL_TOKEN=eyJ-s3cr3t"])));
        assert_eq!(body.user.as_deref(), Some("65532:65532"));
        let host = body.host_config.expect("a host config");
        assert_eq!(host.readonly_rootfs, Some(true));
        assert_eq!(host.mounts, Some(Vec::new()));
        let endpoints = body
            .networking_config
            .and_then(|networking| networking.endpoints_config)
            .expect("endpoints");
        let mut networks: Vec<&String> = endpoints.keys().collect();
        networks.sort();
        assert_eq!(networks, ["brume-ingress", "brume-tunnel"]);
        assert_eq!(
            endpoints["brume-ingress"]
                .ipam_config
                .as_ref()
                .and_then(|ipam| ipam.ipv4_address.as_deref()),
            Some("10.213.255.11")
        );
    }

    #[test]
    fn network_request_pins_the_subnet() {
        let labels = HashMap::from([("brume.managed".to_owned(), "1".to_owned())]);
        let ingress = NetworkSpec {
            name: "brume-ingress".to_owned(),
            internal: true,
            environment_id: String::new(),
            subnet: Some("10.213.255.0/24".to_owned()),
        };
        let request = network_request(&ingress, labels.clone());
        assert_eq!(request.internal, Some(true));
        assert_eq!(request.labels, Some(labels.clone()), "no empty environment");
        assert_eq!(
            request.ipam,
            Some(Ipam {
                config: Some(vec![IpamConfig {
                    subnet: Some("10.213.255.0/24".to_owned()),
                    ..IpamConfig::default()
                }]),
                ..Ipam::default()
            })
        );

        let environment = NetworkSpec {
            name: "brume-env-1".to_owned(),
            internal: true,
            environment_id: "env-1".to_owned(),
            subnet: None,
        };
        let request = network_request(&environment, labels);
        assert_eq!(request.ipam, None, "Docker picks from its pools");
        assert_eq!(
            request
                .labels
                .as_ref()
                .and_then(|labels| labels.get("brume.environment"))
                .map(String::as_str),
            Some("env-1")
        );
    }

    #[test]
    fn listed_network_carries_its_subnets() {
        let network = Network {
            name: Some("brume-ingress".to_owned()),
            internal: Some(true),
            labels: Some(HashMap::from([(
                "brume.managed".to_owned(),
                "1".to_owned(),
            )])),
            ipam: Some(Ipam {
                config: Some(vec![
                    IpamConfig {
                        subnet: Some("10.213.255.0/24".to_owned()),
                        gateway: Some("10.213.255.1".to_owned()),
                        ..IpamConfig::default()
                    },
                    IpamConfig::default(),
                ]),
                ..Ipam::default()
            }),
            ..Network::default()
        };
        let snap = snap_network(network).expect("a named network");
        assert!(snap.managed && snap.internal);
        assert_eq!(snap.subnets, ["10.213.255.0/24"]);
        assert!(snap_network(Network::default()).is_none());
    }

    #[test]
    fn components_keep_their_logs_in_docker() {
        let inspect = ContainerInspectResponse {
            config: Some(DockerConfig {
                labels: Some(HashMap::from([
                    ("brume.managed".to_owned(), "1".to_owned()),
                    ("brume.component".to_owned(), "cloudflared".to_owned()),
                ])),
                ..DockerConfig::default()
            }),
            ..ContainerInspectResponse::default()
        };
        let (container, source) = managed(CLOUDFLARED_CONTAINER.to_owned(), inspect);
        assert!(container.managed);
        assert_eq!(source, None);
    }
}
