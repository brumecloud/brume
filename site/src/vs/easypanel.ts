// Brume vs Easypanel, from archi/jalons/2026-11-01-recherche-vs/easypanel.md.

import type { Competitor } from "./model";

export const easypanel: Competitor = {
  slug: "easypanel",
  name: "Easypanel",
  kind: "self-hosted",
  icon: "easypanel",
  seoTitle: "Brume vs Easypanel: an open source Easypanel alternative",
  description:
    "Easypanel is a closed-source panel with 845 templates. Brume is an open source alpha whose panel stays on your tailnet and never holds the Docker socket.",
  oneLiner: "A closed-source web panel with one-click templates, priced per server.",
  verdict:
    "Easypanel does more today: templates, backups, teams and clusters. Brume is an open source alpha that keeps its panel on your tailnet and away from Docker.",
  rows: [
    {
      key: "panel",
      them: "A web UI, with 2FA on every plan",
      source: "https://easypanel.io/pricing",
    },
    { key: "docker", them: "The panel drives Docker Swarm", source: "https://easypanel.io/docs" },
    {
      key: "hacked",
      them: "Deploy any image, even reboot the host",
      source: "https://easypanel.io/docs/api/server/reboot",
    },
    { key: "ports", them: "80 and 443, for Traefik", source: "https://easypanel.io/docs" },
    {
      key: "agents",
      them: "A CLI and remote MCP server, on paid plans",
      source: "https://easypanel.io/pricing",
    },
    {
      key: "backups",
      them: "Scheduled, to S3 and more, on paid plans",
      source: "https://easypanel.io/pricing",
    },
    {
      key: "servers",
      them: "Worker nodes on Business, in alpha",
      source: "https://easypanel.io/pricing",
    },
    {
      key: "license",
      them: "Proprietary. Free plan, then a license per server",
      source: "https://easypanel.io/pricing",
    },
    {
      key: "maturity",
      them: "Since 2022, 7.7M Docker Hub pulls",
      source: "https://hub.docker.com/r/easypanel/easypanel",
    },
  ],
  different: [
    "Open source and free, with no per-server license.",
    "The panel stays on your tailnet, behind your Tailscale identity and a second factor.",
    "The panel never holds Docker: a separate agent does, under its own policy.",
  ],
  better: [
    "845 one-click templates, plus managed MySQL, MariaDB, Postgres, MongoDB and Redis.",
    "Cloudflare Tunnel and 2FA on every plan, Free included.",
    "Multiple users with project-level access control, on Growth and up.",
  ],
  pickThem: "You want templates, backups and teams today, and a per-server license is fine.",
  pickBrume: "You want open source, and a panel on your tailnet that never touches Docker.",
  checked: "2026-11-01",
};
