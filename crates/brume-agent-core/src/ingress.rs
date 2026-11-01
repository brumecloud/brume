//! The public entry: cloudflared runs the Cloudflare tunnel and hands every request to Traefik,
//! which sends each public host to its service, on the edge network that the service shares
//! with Traefik alone. The agent runs both from the engine's `IngressSpec`, with configs that it
//! builds itself: only the tunnel token and the routes come from the engine, and a route is used
//! only once its host, network, target and port pass `valid_routes`.

use std::collections::{BTreeMap, BTreeSet};

use brume_proto::agent::{EnvValue, EnvVar, IngressRoute, IngressSpec, NetworkSpec};
use serde_json::{Map, Value, json};

use crate::config::{
    BindMount, Component, ContainerConfig, Endpoint, HostConfig, HostDir, LABEL_COMPONENT,
    LABEL_MANAGED, locked_host_config, with_hash,
};

/// Traefik v3.7.17, pinned by the digest of its multi-arch index.
pub const TRAEFIK_IMAGE: &str =
    "traefik:v3.7.17@sha256:30ff62491aacdb862ceb8c9edcf2627da638d7deafdd4b6dab32fe34f8aa3e90";
/// cloudflared 2026.10.7, pinned by the digest of its multi-arch index.
pub const CLOUDFLARED_IMAGE: &str = "cloudflare/cloudflared:2026.10.7@sha256:\
     4955524aeaa379e154023d5a027c250f7b35041a2e7e950cf13f144bc9b9c5ba";

/// The network through which cloudflared reaches Cloudflare: the only one of the entry with a
/// route to the internet.
pub const TUNNEL_NETWORK: &str = "brume-tunnel";
/// The internal network that cloudflared and Traefik share, and nothing else.
pub const INGRESS_NETWORK: &str = "brume-ingress";
/// The subnet of `INGRESS_NETWORK`, outside Docker's address pools.
pub const INGRESS_SUBNET: &str = "10.213.255.0/24";
/// Traefik's address on `INGRESS_NETWORK`, the only one it listens on: a service on an edge
/// network cannot reach it with another service's host.
pub const TRAEFIK_ADDRESS: &str = "10.213.255.10";
/// cloudflared's address on `INGRESS_NETWORK`. Traefik trusts the forwarded headers from it
/// alone.
pub const CLOUDFLARED_ADDRESS: &str = "10.213.255.11";
/// The port of Traefik's entrypoint.
pub const TRAEFIK_PORT: u16 = 8080;

/// The container that runs Traefik.
pub const TRAEFIK_CONTAINER: &str = "brume-traefik";
/// The container that runs cloudflared.
pub const CLOUDFLARED_CONTAINER: &str = "brume-cloudflared";

/// Where Traefik reads the agent's config: `HostDir::TraefikConfig`, mounted read-only.
pub const TRAEFIK_CONFIG_DIR: &str = "/etc/traefik/dynamic";
/// The agent's config for Traefik, in that directory. JSON, which is also YAML.
pub const TRAEFIK_CONFIG_FILE: &str = "dynamic.yml";

/// Traefik's only entrypoint.
const ENTRYPOINT: &str = "web";
/// The prefix of the edge networks.
const EDGE_PREFIX: &str = "brume-edge-";
/// nobody:nogroup.
const TRAEFIK_USER: &str = "65534:65534";
/// The image's own non-root user, set again in case a later image drops it.
const CLOUDFLARED_USER: &str = "65532:65532";
const COMPONENT_CPU_MILLIS: u32 = 1_000;
const COMPONENT_MEMORY_MB: u32 = 256;
const COMPONENT_STOP_TIMEOUT_SECS: u32 = 10;
/// The longest DNS name.
const MAX_HOST_BYTES: usize = 253;
/// The longest DNS label.
const MAX_LABEL_BYTES: usize = 63;

/// Whether `name` is one of the entry's networks, which only its components join: an engine's
/// network or container never uses one.
pub fn is_component_network(name: &str) -> bool {
    name == TUNNEL_NETWORK || name == INGRESS_NETWORK
}

/// The routes that Traefik uses, in order: a host that is a lowercase DNS name of two labels at
/// least, an edge network `brume-edge-[a-z0-9-]+`, a target that is one DNS label, and a port
/// other than 0. A host already routed keeps its first route. The others are left out: a
/// backtick in a host would inject a condition into Traefik's rule.
pub fn valid_routes(routes: &[IngressRoute]) -> Vec<&IngressRoute> {
    let mut hosts = BTreeSet::new();
    routes
        .iter()
        .filter(|route| {
            is_host(&route.host)
                && is_edge_network(&route.network)
                && is_label(&route.target)
                && route.port != 0
        })
        .filter(|route| hosts.insert(route.host.as_str()))
        .collect()
}

/// The networks and containers of the public entry: `brume-tunnel` and `brume-ingress`, then
/// Traefik and cloudflared. Traefik joins `brume-ingress` and the edge network of each valid
/// route; the plan leaves out an edge network that it cannot join yet.
pub fn ingress_components(spec: &IngressSpec) -> (Vec<NetworkSpec>, Vec<ContainerConfig>) {
    let networks = vec![
        NetworkSpec {
            name: TUNNEL_NETWORK.to_owned(),
            internal: false,
            environment_id: String::new(),
            subnet: None,
        },
        NetworkSpec {
            name: INGRESS_NETWORK.to_owned(),
            internal: true,
            environment_id: String::new(),
            subnet: Some(INGRESS_SUBNET.to_owned()),
        },
    ];
    let containers = vec![traefik(&spec.routes), cloudflared(spec)];
    (networks, containers)
}

/// Traefik's dynamic config: for each valid route, a router on the `web` entrypoint whose rule
/// is ``Host(`<host>`)`` and a service at `http://<target>:<port>`, both named after the host.
/// A request for any other host gets Traefik's 404. Without a route, an empty object: Traefik
/// refuses empty `routers` and `services`, and a config that it refuses at its start is never
/// watched again.
pub fn traefik_dynamic_config(routes: &[IngressRoute]) -> Value {
    let mut routers = Map::new();
    let mut services = Map::new();
    for route in valid_routes(routes) {
        // Injective, as a valid host has no `_`, and free of the `.` and `@` that Traefik reads
        // in a name.
        let name = route.host.replace('.', "_");
        routers.insert(
            name.clone(),
            json!({
                "rule": format!("Host(`{}`)", route.host),
                "entryPoints": [ENTRYPOINT],
                "service": name,
            }),
        );
        services.insert(
            name,
            json!({
                "loadBalancer": {
                    "servers": [{ "url": format!("http://{}:{}", route.target, route.port) }],
                },
            }),
        );
    }
    if routers.is_empty() {
        return json!({});
    }
    json!({ "http": { "routers": routers, "services": services } })
}

/// Traefik: on `brume-ingress` at its fixed address, and on each route's edge network; nobody,
/// a read-only root, the agent's config mounted read-only.
fn traefik(routes: &[IngressRoute]) -> ContainerConfig {
    let mut endpoints = vec![Endpoint {
        network: INGRESS_NETWORK.to_owned(),
        aliases: Vec::new(),
        ipv4_address: Some(TRAEFIK_ADDRESS.to_owned()),
    }];
    let mut joined = BTreeSet::new();
    for route in valid_routes(routes) {
        if joined.insert(route.network.as_str()) {
            endpoints.push(Endpoint {
                network: route.network.clone(),
                aliases: Vec::new(),
                ipv4_address: None,
            });
        }
    }
    let host_config = HostConfig {
        binds: vec![BindMount {
            source: HostDir::TraefikConfig,
            target: TRAEFIK_CONFIG_DIR.to_owned(),
        }],
        ..component_host_config(&endpoints)
    };
    let cmd = vec![
        format!("--entryPoints.{ENTRYPOINT}.address={TRAEFIK_ADDRESS}:{TRAEFIK_PORT}"),
        format!("--entryPoints.{ENTRYPOINT}.forwardedHeaders.trustedIPs={CLOUDFLARED_ADDRESS}/32"),
        // A header that aliases one that Traefik manages, such as `X_Forwarded_For`, reads as
        // that header in nginx, PHP or CGI: it goes, so that a client cannot spoof one.
        format!("--entryPoints.{ENTRYPOINT}.http.aliasHeadersStrategy=delete"),
        format!("--providers.file.filename={TRAEFIK_CONFIG_DIR}/{TRAEFIK_CONFIG_FILE}"),
        "--providers.file.watch=true".to_owned(),
        "--accessLog=true".to_owned(),
        "--log.level=INFO".to_owned(),
        "--global.checkNewVersion=false".to_owned(),
        "--global.sendAnonymousUsage=false".to_owned(),
    ];
    with_hash(ContainerConfig {
        name: TRAEFIK_CONTAINER.to_owned(),
        image: TRAEFIK_IMAGE.to_owned(),
        env: Vec::new(),
        skipped_env: 0,
        cmd: Some(cmd),
        user: Some(TRAEFIK_USER.to_owned()),
        labels: labels(Component::Traefik),
        stop_timeout_secs: COMPONENT_STOP_TIMEOUT_SECS,
        host_config,
        endpoints,
        component: Some(Component::Traefik),
    })
}

/// cloudflared: on `brume-ingress` at its fixed address, and on `brume-tunnel` for Cloudflare;
/// its token in its environment, every request to Traefik.
fn cloudflared(spec: &IngressSpec) -> ContainerConfig {
    let endpoints = vec![
        Endpoint {
            network: INGRESS_NETWORK.to_owned(),
            aliases: Vec::new(),
            ipv4_address: Some(CLOUDFLARED_ADDRESS.to_owned()),
        },
        Endpoint {
            network: TUNNEL_NETWORK.to_owned(),
            aliases: Vec::new(),
            ipv4_address: None,
        },
    ];
    let env = vec![EnvVar {
        name: "TUNNEL_TOKEN".to_owned(),
        value: EnvValue::Secret(spec.tunnel_token.clone()),
    }];
    let cmd = vec![
        "tunnel".to_owned(),
        "--no-autoupdate".to_owned(),
        "run".to_owned(),
        "--url".to_owned(),
        format!("http://{TRAEFIK_ADDRESS}:{TRAEFIK_PORT}"),
    ];
    with_hash(ContainerConfig {
        name: CLOUDFLARED_CONTAINER.to_owned(),
        image: CLOUDFLARED_IMAGE.to_owned(),
        env,
        skipped_env: 0,
        cmd: Some(cmd),
        user: Some(CLOUDFLARED_USER.to_owned()),
        labels: labels(Component::Cloudflared),
        stop_timeout_secs: COMPONENT_STOP_TIMEOUT_SECS,
        host_config: component_host_config(&endpoints),
        endpoints,
        component: Some(Component::Cloudflared),
    })
}

/// The `HostConfig` of a component: an app's, without any capability, and with a read-only
/// root.
fn component_host_config(endpoints: &[Endpoint]) -> HostConfig {
    HostConfig {
        readonly_rootfs: true,
        ..locked_host_config(endpoints, COMPONENT_CPU_MILLIS, COMPONENT_MEMORY_MB)
    }
}

/// The labels of a component's container.
fn labels(component: Component) -> BTreeMap<String, String> {
    BTreeMap::from([
        (LABEL_MANAGED.to_owned(), "1".to_owned()),
        (LABEL_COMPONENT.to_owned(), component.label().to_owned()),
    ])
}

/// Whether `host` is a lowercase DNS name of two labels at least.
fn is_host(host: &str) -> bool {
    host.len() <= MAX_HOST_BYTES && host.split('.').count() >= 2 && host.split('.').all(is_label)
}

/// Whether `label` is one DNS label: 1 to 63 of `[a-z0-9-]`, neither first nor last a `-`.
fn is_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= MAX_LABEL_BYTES
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Whether `name` is an edge network, `brume-edge-[a-z0-9-]+`.
fn is_edge_network(name: &str) -> bool {
    name.strip_prefix(EDGE_PREFIX).is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    })
}
