//! The public entry: the complete configs of Traefik and cloudflared, Traefik's routes, and the
//! plans that run, change and remove them.

use std::collections::BTreeMap;

use brume_agent_core::backoff::Restarts;
use brume_agent_core::config::{
    BindMount, Component, ContainerConfig, Endpoint, HostConfig, HostDir, LogConfig, RestartPolicy,
};
use brume_agent_core::health::ProbeResults;
use brume_agent_core::ingress::{
    CLOUDFLARED_CONTAINER, CLOUDFLARED_IMAGE, INGRESS_NETWORK, TRAEFIK_CONTAINER, TRAEFIK_IMAGE,
    TUNNEL_NETWORK, ingress_components, traefik_dynamic_config, valid_routes,
};
use brume_agent_core::observed::{DockerSnapshot, SnapContainer, SnapNetwork, SnapState};
use brume_agent_core::plan::{Action, Refusal, RefusalReason, Resource, plan, refusals};
use brume_proto::agent::{
    ContainerSpec, DesiredState, EnvValue, EnvVar, Healthcheck, IngressRoute, IngressSpec,
    NetworkAttachment, NetworkSpec,
};
use secrecy::SecretString;
use serde_json::json;

const TOKEN: &str = "eyJhIjoiYWNjb3VudCJ9-s3cr3t";
const NOW_MS: u64 = 1_000_000;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

/// A route to the service on `network`, by the network's name, as the engine gives it: Traefik
/// is on every edge network, so a slug could name another environment's service.
fn route(host: &str, network: &str, port: u16) -> IngressRoute {
    route_to(host, network, network, port)
}

fn route_to(host: &str, network: &str, target: &str, port: u16) -> IngressRoute {
    IngressRoute {
        host: host.to_owned(),
        network: network.to_owned(),
        target: target.to_owned(),
        port,
    }
}

/// Two services with a domain: one on 8080 and one on 3000.
fn two_routes() -> Vec<IngressRoute> {
    vec![
        route("demo.example.com", "brume-edge-svc-1", 8080),
        route("api.example.com", "brume-edge-svc-2", 3000),
    ]
}

fn spec(token: &str, routes: Vec<IngressRoute>) -> IngressSpec {
    IngressSpec {
        tunnel_token: SecretString::from(token),
        routes,
    }
}

fn component(spec: &IngressSpec, name: &str) -> ContainerConfig {
    let (_, containers) = ingress_components(spec);
    containers
        .into_iter()
        .find(|config| config.name == name)
        .expect("the component")
}

fn endpoint(network: &str, address: Option<&str>) -> Endpoint {
    Endpoint {
        network: network.to_owned(),
        aliases: Vec::new(),
        ipv4_address: address.map(ToOwned::to_owned),
    }
}

/// The `HostConfig` of both components, on `network_mode`.
fn component_host_config(network_mode: &str, binds: Vec<BindMount>) -> HostConfig {
    HostConfig {
        network_mode: network_mode.to_owned(),
        cap_drop: strings(&["ALL"]),
        cap_add: Vec::new(),
        security_opt: strings(&["no-new-privileges:true"]),
        memory_bytes: 256 * 1024 * 1024,
        memory_swap_bytes: 256 * 1024 * 1024,
        nano_cpus: 1_000_000_000,
        pids_limit: 512,
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
        readonly_rootfs: true,
        binds,
    }
}

#[test]
fn traefik_full_config() {
    let expected = ContainerConfig {
        name: "brume-traefik".to_owned(),
        image: "traefik:v3.7.17@sha256:\
                30ff62491aacdb862ceb8c9edcf2627da638d7deafdd4b6dab32fe34f8aa3e90"
            .to_owned(),
        env: Vec::new(),
        skipped_env: 0,
        cmd: Some(strings(&[
            "--entryPoints.web.address=10.213.255.10:8080",
            "--entryPoints.web.forwardedHeaders.trustedIPs=10.213.255.11/32",
            "--entryPoints.web.http.aliasHeadersStrategy=delete",
            "--providers.file.filename=/etc/traefik/dynamic/dynamic.yml",
            "--providers.file.watch=true",
            "--accessLog=true",
            "--log.level=INFO",
            "--global.checkNewVersion=false",
            "--global.sendAnonymousUsage=false",
        ])),
        user: Some("65534:65534".to_owned()),
        labels: BTreeMap::from([
            ("brume.component".to_owned(), "traefik".to_owned()),
            // Pinned: a change of the config replaces Traefik.
            (
                "brume.config-hash".to_owned(),
                "96146829ef3ad8a9c8ce5cb9ed664384572992b4b670150c3e49f1aba731f9b2".to_owned(),
            ),
            ("brume.managed".to_owned(), "1".to_owned()),
        ]),
        stop_timeout_secs: 10,
        host_config: component_host_config(
            "brume-ingress",
            vec![BindMount {
                source: HostDir::TraefikConfig,
                target: "/etc/traefik/dynamic".to_owned(),
            }],
        ),
        endpoints: vec![
            endpoint("brume-ingress", Some("10.213.255.10")),
            endpoint("brume-edge-svc-1", None),
            endpoint("brume-edge-svc-2", None),
        ],
        component: Some(Component::Traefik),
    };
    assert_eq!(
        component(&spec(TOKEN, two_routes()), TRAEFIK_CONTAINER),
        expected
    );
}

#[test]
fn cloudflared_full_config() {
    let expected = ContainerConfig {
        name: "brume-cloudflared".to_owned(),
        image: "cloudflare/cloudflared:2026.10.7@sha256:\
                4955524aeaa379e154023d5a027c250f7b35041a2e7e950cf13f144bc9b9c5ba"
            .to_owned(),
        env: vec![EnvVar {
            name: "TUNNEL_TOKEN".to_owned(),
            value: EnvValue::Secret(SecretString::from(TOKEN)),
        }],
        skipped_env: 0,
        cmd: Some(strings(&[
            "tunnel",
            "--no-autoupdate",
            "run",
            "--url",
            "http://10.213.255.10:8080",
        ])),
        user: Some("65532:65532".to_owned()),
        labels: BTreeMap::from([
            ("brume.component".to_owned(), "cloudflared".to_owned()),
            // Pinned: a change of the config, or of the token, replaces cloudflared.
            (
                "brume.config-hash".to_owned(),
                "064d079cf739dc8bd06b0b6621f717c1f93f5c85b5cca6fdee177ca72612682c".to_owned(),
            ),
            ("brume.managed".to_owned(), "1".to_owned()),
        ]),
        stop_timeout_secs: 10,
        host_config: component_host_config("brume-ingress", Vec::new()),
        endpoints: vec![
            endpoint("brume-ingress", Some("10.213.255.11")),
            endpoint("brume-tunnel", None),
        ],
        component: Some(Component::Cloudflared),
    };
    let config = component(&spec(TOKEN, two_routes()), CLOUDFLARED_CONTAINER);
    assert_eq!(config, expected);
    // The token stays masked, and out of the labels.
    assert!(!format!("{config:?}").contains("s3cr3t"));
    assert!(
        config
            .labels
            .values()
            .all(|value| !value.contains("s3cr3t"))
    );
    // Its routes are Traefik's business.
    assert_eq!(
        component(&spec(TOKEN, Vec::new()), CLOUDFLARED_CONTAINER),
        config
    );
}

#[test]
fn entry_networks_are_the_tunnel_and_a_fixed_ingress_subnet() {
    let (networks, _) = ingress_components(&spec(TOKEN, two_routes()));
    assert_eq!(
        networks,
        [
            NetworkSpec {
                name: "brume-tunnel".to_owned(),
                internal: false,
                environment_id: String::new(),
                subnet: None,
            },
            NetworkSpec {
                name: "brume-ingress".to_owned(),
                internal: true,
                environment_id: String::new(),
                subnet: Some("10.213.255.0/24".to_owned()),
            },
        ]
    );
}

#[test]
fn dynamic_config_routes_hosts() {
    assert_eq!(
        traefik_dynamic_config(&two_routes()),
        json!({
            "http": {
                "routers": {
                    "demo_example_com": {
                        "rule": "Host(`demo.example.com`)",
                        "entryPoints": ["web"],
                        "service": "demo_example_com",
                    },
                    "api_example_com": {
                        "rule": "Host(`api.example.com`)",
                        "entryPoints": ["web"],
                        "service": "api_example_com",
                    },
                },
                "services": {
                    "demo_example_com": {
                        "loadBalancer": { "servers": [{ "url": "http://brume-edge-svc-1:8080" }] },
                    },
                    "api_example_com": {
                        "loadBalancer": { "servers": [{ "url": "http://brume-edge-svc-2:3000" }] },
                    },
                },
            },
        })
    );
    // No route: every host gets Traefik's 404. Traefik refuses empty routers and services.
    assert_eq!(traefik_dynamic_config(&[]), json!({}));
}

#[test]
fn invalid_routes_are_left_out() {
    let routes = vec![
        route("demo.example.com", "brume-edge-svc-1", 8080),
        // An injection through the rule.
        route("x`) || PathPrefix(`/", "brume-edge-svc-2", 8080),
        route("Demo.example.com", "brume-edge-svc-2", 8080),
        route("localhost", "brume-edge-svc-2", 8080),
        route("-x.example.com", "brume-edge-svc-2", 8080),
        route(
            &format!("{}.example.com", "a".repeat(64)),
            "brume-edge-svc-2",
            8080,
        ),
        // Traefik never joins an environment network, where the databases are.
        route("db.example.com", "brume-env-1", 8080),
        route("host.example.com", "host", 8080),
        route_to("t.example.com", "brume-edge-svc-2", "web/../x", 8080),
        route_to("u.example.com", "brume-edge-svc-2", "web@evil", 8080),
        route("zero.example.com", "brume-edge-svc-2", 0),
        // A host already routed keeps its first route.
        route("demo.example.com", "brume-edge-svc-3", 80),
        route("ok.example.com", "brume-edge-svc-4", 80),
    ];
    let valid: Vec<&str> = valid_routes(&routes)
        .iter()
        .map(|route| route.host.as_str())
        .collect();
    assert_eq!(valid, ["demo.example.com", "ok.example.com"]);
    let config = traefik_dynamic_config(&routes);
    let routers = config["http"]["routers"].as_object().expect("routers");
    assert_eq!(routers.len(), 2);
    assert_eq!(
        config["http"]["services"]["demo_example_com"]["loadBalancer"]["servers"][0]["url"],
        "http://brume-edge-svc-1:8080"
    );
    let traefik = component(&spec(TOKEN, routes), TRAEFIK_CONTAINER);
    let networks: Vec<&str> = traefik
        .endpoints
        .iter()
        .map(|endpoint| endpoint.network.as_str())
        .collect();
    assert_eq!(
        networks,
        ["brume-ingress", "brume-edge-svc-1", "brume-edge-svc-4"]
    );
}

// --- Plans. ---

/// An app without networks.
fn app() -> ContainerSpec {
    ContainerSpec {
        name: "brume-web-0190a3f2".to_owned(),
        image: "nginx@sha256:aa".to_owned(),
        environment_id: "env".to_owned(),
        service_id: "svc".to_owned(),
        deployment_id: "dep".to_owned(),
        env: Vec::new(),
        command: None,
        networks: Vec::new(),
        mounts: Vec::new(),
        cpu_millis: 100,
        memory_mb: 64,
        stop_timeout_secs: 10,
        healthcheck: Healthcheck::Running { grace_secs: 0 },
    }
}

fn network(name: &str, internal: bool) -> NetworkSpec {
    NetworkSpec {
        name: name.to_owned(),
        internal,
        environment_id: "env".to_owned(),
        subnet: None,
    }
}

/// The engine's desired state for `routes`: their edge networks, and the entry.
fn desired(ingress: Option<IngressSpec>) -> DesiredState {
    let networks = ingress
        .iter()
        .flat_map(|ingress| &ingress.routes)
        .map(|route| network(&route.network, true))
        .collect();
    DesiredState {
        position: 1,
        networks,
        volumes: Vec::new(),
        containers: Vec::new(),
        ingress,
    }
}

fn snap_network(name: &str, internal: bool, subnet: Option<&str>) -> SnapNetwork {
    SnapNetwork {
        name: name.to_owned(),
        internal,
        managed: true,
        subnets: subnet.into_iter().map(ToOwned::to_owned).collect(),
    }
}

/// What Docker runs once `config` ran for a while.
fn running(config: &ContainerConfig) -> SnapContainer {
    SnapContainer {
        name: config.name.clone(),
        managed: true,
        deployment_id: String::new(),
        image: config.image.clone(),
        state: SnapState::Running,
        started_at_ms: Some(NOW_MS - 120_000),
        networks: config
            .endpoints
            .iter()
            .map(|endpoint| NetworkAttachment {
                network: endpoint.network.clone(),
                aliases: endpoint.aliases.clone(),
            })
            .collect(),
        mounts: Vec::new(),
        config_hash: config.config_hash().to_owned(),
        stop_timeout_secs: config.stop_timeout_secs,
    }
}

/// Docker with the entry of `spec` running, its networks and images present.
fn docker_with(spec: &IngressSpec) -> DockerSnapshot {
    let (_, containers) = ingress_components(spec);
    let mut networks = vec![
        snap_network(TUNNEL_NETWORK, false, Some("10.213.1.0/24")),
        snap_network(INGRESS_NETWORK, true, Some("10.213.255.0/24")),
    ];
    for route in &spec.routes {
        networks.push(snap_network(&route.network, true, Some("10.213.2.0/24")));
    }
    DockerSnapshot {
        networks,
        images: vec![
            "traefik@sha256:30ff62491aacdb862ceb8c9edcf2627da638d7deafdd4b6dab32fe34f8aa3e90"
                .to_owned(),
            "cloudflare/cloudflared@sha256:\
             4955524aeaa379e154023d5a027c250f7b35041a2e7e950cf13f144bc9b9c5ba"
                .to_owned(),
        ],
        containers: containers.iter().map(running).collect(),
        ..DockerSnapshot::default()
    }
}

fn plan_now(desired: &DesiredState, snapshot: &DockerSnapshot) -> Vec<Action> {
    plan(
        desired,
        snapshot,
        &Restarts::default(),
        &ProbeResults::default(),
        NOW_MS,
    )
}

/// The names of the containers that `actions` create.
fn created(actions: &[Action]) -> Vec<&str> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::CreateContainer(config) => Some(config.name.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn entry_starts_from_scratch_in_two_rounds() {
    let spec = spec(TOKEN, two_routes());
    let desired = desired(Some(spec.clone()));
    let actions = plan_now(&desired, &DockerSnapshot::default());
    let networks: Vec<&NetworkSpec> = actions
        .iter()
        .filter_map(|action| match action {
            Action::CreateNetwork(network) => Some(network),
            _ => None,
        })
        .collect();
    let names: Vec<&str> = networks
        .iter()
        .map(|network| network.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "brume-edge-svc-1",
            "brume-edge-svc-2",
            "brume-tunnel",
            "brume-ingress"
        ]
    );
    assert_eq!(networks[3].subnet.as_deref(), Some("10.213.255.0/24"));
    assert!(created(&actions).is_empty(), "the networks first");

    // The networks are there: both components, with their images.
    let mut snapshot = docker_with(&spec);
    snapshot.containers.clear();
    snapshot.images.clear();
    let actions = plan_now(&desired, &snapshot);
    assert_eq!(
        created(&actions),
        [TRAEFIK_CONTAINER, CLOUDFLARED_CONTAINER]
    );
    assert!(actions.contains(&Action::PullImage(TRAEFIK_IMAGE.to_owned())));
    assert!(actions.contains(&Action::PullImage(CLOUDFLARED_IMAGE.to_owned())));

    // Running: nothing left to do. The images match without their tags.
    assert_eq!(plan_now(&desired, &docker_with(&spec)), []);
}

#[test]
fn no_ingress_no_components() {
    // Nothing to run.
    let actions = plan_now(&desired(None), &DockerSnapshot::default());
    assert_eq!(actions, []);

    // A running entry goes: both containers stop and go. Its networks stay, as other unused
    // networks do.
    let snapshot = docker_with(&spec(TOKEN, two_routes()));
    let actions = plan_now(&desired(None), &snapshot);
    assert_eq!(
        actions,
        [
            Action::StopContainer {
                name: TRAEFIK_CONTAINER.to_owned(),
                timeout_secs: 10,
            },
            Action::RemoveContainer(TRAEFIK_CONTAINER.to_owned()),
            Action::StopContainer {
                name: CLOUDFLARED_CONTAINER.to_owned(),
                timeout_secs: 10,
            },
            Action::RemoveContainer(CLOUDFLARED_CONTAINER.to_owned()),
        ]
    );
}

#[test]
fn token_change_recreates_cloudflared() {
    let before = spec(TOKEN, two_routes());
    let after = spec("eyJhIjoiYWNjb3VudCJ9-rotated", two_routes());
    let snapshot = docker_with(&before);
    let actions = plan_now(&desired(Some(after.clone())), &snapshot);
    assert_eq!(
        actions,
        [
            Action::StopContainer {
                name: CLOUDFLARED_CONTAINER.to_owned(),
                timeout_secs: 10,
            },
            Action::RemoveContainer(CLOUDFLARED_CONTAINER.to_owned()),
        ]
    );
    let old = component(&before, CLOUDFLARED_CONTAINER);
    let new = component(&after, CLOUDFLARED_CONTAINER);
    assert_ne!(old.config_hash(), new.config_hash());

    // Gone: it comes back with the new token.
    let mut gone = snapshot;
    gone.containers
        .retain(|container| container.name != CLOUDFLARED_CONTAINER);
    let actions = plan_now(&desired(Some(after)), &gone);
    assert_eq!(created(&actions), [CLOUDFLARED_CONTAINER]);
    assert!(actions.contains(&Action::CreateContainer(Box::new(new))));
}

#[test]
fn route_change_keeps_traefik() {
    let before = spec(TOKEN, two_routes());
    let snapshot = docker_with(&before);

    // A route to another port of the same service: only the file changes.
    let mut routes = two_routes();
    routes[0].port = 9090;
    let moved = spec(TOKEN, routes);
    assert_eq!(plan_now(&desired(Some(moved.clone())), &snapshot), []);
    assert_ne!(
        traefik_dynamic_config(&moved.routes),
        traefik_dynamic_config(&before.routes)
    );

    // A third service: Traefik joins its edge network live, once it exists.
    let mut routes = two_routes();
    routes.push(route("new.example.com", "brume-edge-svc-3", 80));
    let grown = spec(TOKEN, routes);
    let actions = plan_now(&desired(Some(grown.clone())), &snapshot);
    assert_eq!(
        actions,
        [Action::CreateNetwork(network("brume-edge-svc-3", true))]
    );
    let mut with_network = snapshot.clone();
    with_network.networks.push(snap_network(
        "brume-edge-svc-3",
        true,
        Some("10.213.3.0/24"),
    ));
    assert_eq!(
        plan_now(&desired(Some(grown)), &with_network),
        [Action::Reattach {
            container: TRAEFIK_CONTAINER.to_owned(),
            network: "brume-edge-svc-3".to_owned(),
            aliases: Vec::new(),
            ipv4_address: None,
        }]
    );

    // A service loses its domain: Traefik leaves its edge network live.
    let shrunk = spec(TOKEN, two_routes()[..1].to_vec());
    assert_eq!(
        plan_now(&desired(Some(shrunk)), &snapshot),
        [Action::Disconnect {
            container: TRAEFIK_CONTAINER.to_owned(),
            network: "brume-edge-svc-2".to_owned(),
        }]
    );
}

#[test]
fn network_with_another_subnet_is_created_again() {
    let spec = spec(TOKEN, two_routes());
    let desired = desired(Some(spec.clone()));
    let mut snapshot = docker_with(&spec);
    snapshot.networks[1].subnets = strings(&["172.30.0.0/16"]);
    let actions = plan_now(&desired, &snapshot);
    assert_eq!(
        actions,
        [
            Action::StopContainer {
                name: TRAEFIK_CONTAINER.to_owned(),
                timeout_secs: 10,
            },
            Action::RemoveContainer(TRAEFIK_CONTAINER.to_owned()),
            Action::StopContainer {
                name: CLOUDFLARED_CONTAINER.to_owned(),
                timeout_secs: 10,
            },
            Action::RemoveContainer(CLOUDFLARED_CONTAINER.to_owned()),
            Action::RemoveNetwork(INGRESS_NETWORK.to_owned()),
        ]
    );

    // Gone with its containers: created again, then they follow.
    snapshot.containers.clear();
    snapshot.networks.remove(1);
    let actions = plan_now(&desired, &snapshot);
    assert!(matches!(
        actions.as_slice(),
        [Action::CreateNetwork(network)] if network.name == INGRESS_NETWORK
    ));
}

#[test]
fn engine_cannot_use_the_entry_networks() {
    // An engine that declares the ingress network and puts an app on it, to reach Traefik
    // without Cloudflare.
    let on_ingress = ContainerSpec {
        networks: vec![NetworkAttachment {
            network: INGRESS_NETWORK.to_owned(),
            aliases: strings(&["web"]),
        }],
        ..app()
    };
    let mut state = desired(None);
    state.networks.push(network(INGRESS_NETWORK, false));
    state.containers.push(on_ingress.clone());
    let mut snapshot = DockerSnapshot::default();
    assert_eq!(plan_now(&state, &snapshot), []);
    assert_eq!(
        refusals(&state, &snapshot),
        [Refusal {
            resource: Resource::Network,
            name: INGRESS_NETWORK.to_owned(),
            reason: RefusalReason::Reserved,
        }]
    );
    // Even once the network exists.
    snapshot
        .networks
        .push(snap_network(INGRESS_NETWORK, true, Some("10.213.255.0/24")));
    assert_eq!(plan_now(&state, &snapshot), []);

    // Nor can it put an app on them once the entry has them.
    let entry = spec(TOKEN, two_routes());
    let mut state = desired(Some(entry.clone()));
    state.containers.push(on_ingress);
    state.containers.push(ContainerSpec {
        name: "brume-out-0190a3f2".to_owned(),
        networks: vec![NetworkAttachment {
            network: TUNNEL_NETWORK.to_owned(),
            aliases: Vec::new(),
        }],
        ..app()
    });
    assert_eq!(plan_now(&state, &docker_with(&entry)), []);

    // Nor can it take a component's name.
    let mut state = desired(Some(entry.clone()));
    state.containers.push(ContainerSpec {
        name: TRAEFIK_CONTAINER.to_owned(),
        ..app()
    });
    assert_eq!(plan_now(&state, &docker_with(&entry)), []);
}

#[test]
fn traefik_skips_an_edge_network_it_cannot_join() {
    let spec = spec(TOKEN, two_routes());
    let desired = desired(Some(spec.clone()));
    let mut snapshot = docker_with(&spec);
    snapshot.containers.clear();
    // Someone else's network under the second service's edge name.
    snapshot.networks[3].managed = false;
    let actions = plan_now(&desired, &snapshot);
    let traefik = actions
        .iter()
        .find_map(|action| match action {
            Action::CreateContainer(config) if config.name == TRAEFIK_CONTAINER => Some(config),
            _ => None,
        })
        .expect("Traefik is created");
    let networks: Vec<&str> = traefik
        .endpoints
        .iter()
        .map(|endpoint| endpoint.network.as_str())
        .collect();
    assert_eq!(networks, ["brume-ingress", "brume-edge-svc-1"]);
    assert!(refusals(&desired, &snapshot).contains(&Refusal {
        resource: Resource::Network,
        name: "brume-edge-svc-2".to_owned(),
        reason: RefusalReason::Foreign,
    }));
}
