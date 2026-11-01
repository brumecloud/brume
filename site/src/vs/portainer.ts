// Brume vs Portainer. Facts from archi/jalons/2026-11-01-recherche-vs/portainer.md. Portainer is a
// container manager rather than a deploy platform, so some rows compare different jobs.

import type { Competitor } from "./model";

export const portainer: Competitor = {
  slug: "portainer",
  name: "Portainer",
  kind: "self-hosted",
  icon: "portainer",
  seoTitle: "Brume vs Portainer: a Portainer alternative for app deploys",
  description:
    "Portainer manages containers across Docker, Swarm and Kubernetes. Brume deploys your code to one server, with its panel on your tailnet, away from Docker.",
  oneLiner: "A container manager for Docker, Swarm and Kubernetes fleets.",
  verdict:
    "Portainer is a mature manager for many Docker and Kubernetes environments. Brume does less: it builds and deploys your apps on one server, with the panel off the internet and away from Docker.",
  rows: [
    {
      key: "panel",
      them: "Public on port 9443, over HTTPS",
      source: "https://docs.portainer.io/start/install-ce/server/docker/linux",
    },
    {
      key: "docker",
      them: "Docker socket mounted in the UI's container",
      source: "https://docs.portainer.io/start/install-ce/server/docker/linux",
    },
    {
      key: "hacked",
      them: "Near root-level control of your infrastructure",
      source:
        "https://www.portainer.io/how-to/how-to-correctly-secure-portainer-when-presented-on-the-internet",
    },
    {
      key: "ports",
      them: "9443, plus 8000 for Edge agents",
      source: "https://docs.portainer.io/start/install-ce/server/docker/linux",
    },
    {
      key: "install",
      them: "One docker run command",
      source: "https://docs.portainer.io/start/install-ce/server/docker/linux",
    },
    {
      key: "agents",
      them: "No CLI. An MCP gateway in Business Edition",
      source: "https://github.com/portainer/portainer-mcp",
    },
    {
      key: "backups",
      them: "Its own config only, not your apps' data",
      source: "https://docs.portainer.io/admin/settings/general",
    },
    {
      key: "servers",
      them: "Docker, Swarm, Podman, Kubernetes and ACI environments",
      source: "https://github.com/portainer/portainer",
    },
    {
      key: "license",
      them: "zlib (CE). Business Edition free for 3 nodes",
      source: "https://github.com/portainer/portainer",
    },
    {
      key: "maturity",
      them: "Releases since 2016, 38k GitHub stars",
      source: "https://github.com/portainer/portainer/releases",
    },
  ],
  different: [
    "Push your code: Brume builds it, ships it and puts it online.",
    "The panel has no Docker socket. A separate agent holds it.",
    "Database backups are on by default, encrypted to your S3.",
  ],
  better: [
    "One UI for many hosts and Kubernetes clusters, its core use case.",
    "Edge agents manage remote hosts with no inbound ports.",
    "Ten years of releases, LDAP and OAuth sign-in, and paid support.",
  ],
  pickThem: "You manage many hosts or Kubernetes clusters and want one container UI.",
  pickBrume: "You want to push code to one server and keep its panel off the internet.",
  checked: "2026-11-01",
};
