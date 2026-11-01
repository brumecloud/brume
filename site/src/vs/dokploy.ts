// Brume vs Dokploy. Facts from archi/jalons/2026-11-01-recherche-vs/dokploy.md.

import type { Competitor } from "./model";

export const dokploy: Competitor = {
  slug: "dokploy",
  name: "Dokploy",
  kind: "self-hosted",
  icon: "dokploy",
  seoTitle: "Brume vs Dokploy: a Dokploy alternative with no open ports",
  description:
    "Dokploy spans many servers and ships an MCP server, and its panel holds the Docker socket. Brume is a one-server alpha with a private panel and no open ports.",
  oneLiner: "A fast-growing Swarm PaaS whose panel holds the Docker socket.",
  verdict:
    "Dokploy has more today: several servers, 590+ templates, a CLI and an MCP server. Brume is a one-server alpha whose panel stays on your tailnet and never holds the Docker socket.",
  rows: [
    {
      key: "panel",
      them: "Port 3000, or Tailscale-only by hand",
      source: "https://docs.dokploy.com/docs/core/guides/tailscale",
    },
    {
      key: "docker",
      them: "Docker socket mounted read-write in the panel",
      source: "https://dokploy.com/install.sh",
    },
    {
      key: "hacked",
      them: "Root on the host, through the Docker socket",
      source: "https://docs.docker.com/engine/security/",
    },
    {
      key: "ports",
      them: "80, 443 and 3000",
      source: "https://docs.dokploy.com/docs/core/installation",
    },
    {
      key: "install",
      them: "One command as root, which sets up Docker Swarm",
      source: "https://dokploy.com/install.sh",
    },
    {
      key: "agents",
      them: "A CLI and an MCP server with 508 tools",
      source: "https://github.com/Dokploy/mcp",
    },
    {
      key: "backups",
      them: "Scheduled database backups to S3",
      source: "https://docs.dokploy.com/docs/core/databases/backups",
    },
    {
      key: "servers",
      them: "Remote servers, build servers and Swarm nodes",
      source: "https://docs.dokploy.com/docs/core/remote-servers",
    },
    {
      key: "license",
      them: "Apache 2.0; SSO and audit logs under a separate license",
      source: "https://github.com/Dokploy/dokploy/blob/canary/LICENSE.MD",
    },
    {
      key: "maturity",
      them: "Releases since 2024, 38k GitHub stars",
      source: "https://github.com/Dokploy/dokploy/releases",
    },
  ],
  different: [
    "The panel never holds the Docker socket. A separate agent does.",
    "Private panel and zero open ports from the install, not from a guide.",
    "An AI agent's irreversible actions wait for your approval.",
  ],
  better: [
    "Remote servers, build servers and Docker Swarm, from one panel.",
    "593 templates and native Docker Compose support.",
    "Passkeys and TOTP for every user. SSO, SCIM and audit logs in Enterprise.",
  ],
  pickThem: "You need several servers, Compose stacks or SSO today.",
  pickBrume:
    "One server is enough, and you want a private panel and closed ports without setting them up by hand.",
  checked: "2026-11-01",
};
