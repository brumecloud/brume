// Brume vs Coolify. Facts from archi/jalons/2026-11-01-recherche-vs/coolify.md.

import type { Competitor } from "./model";

export const coolify: Competitor = {
  slug: "coolify",
  name: "Coolify",
  kind: "self-hosted",
  icon: "coolify",
  seoTitle: "Brume vs Coolify: a Coolify alternative without root SSH",
  description:
    "Coolify drives every server over root SSH, its own host included. Brume is a one-server alpha: the panel stays on your tailnet and only an agent holds Docker.",
  oneLiner: "A mature, multi-server PaaS that drives each server over root SSH.",
  verdict:
    "Coolify is the broader, proven option: many servers, 360+ one-click services, teams and an MCP server. Brume is a one-server alpha whose panel stays off the internet and holds no root SSH key.",
  rows: [
    {
      key: "panel",
      them: "Public on port 8000 until you close it",
      source: "https://coolify.io/docs/knowledge-base/server/firewall",
    },
    {
      key: "docker",
      them: "Root SSH, even locally. The app holds the key",
      source: "https://cdn.coollabs.io/coolify/install.sh",
    },
    {
      key: "hacked",
      them: "Root SSH access to every managed server",
      source: "https://coolify.io/docs/core/security-model",
    },
    {
      key: "ports",
      them: "22, 80, 443, plus 8000, 6001, 6002 at first",
      source: "https://coolify.io/docs/knowledge-base/server/firewall",
    },
    {
      key: "install",
      them: "One command as root, on Linux",
      source: "https://coolify.io/docs/get-started/installation",
    },
    {
      key: "agents",
      them: "A CLI and a built-in MCP server, scoped tokens",
      source: "https://coolify.io/docs/mcp/setup",
    },
    {
      key: "backups",
      them: "Scheduled dumps of six database engines, S3 upload, restore",
      source: "https://coolify.io/docs/databases/backups",
    },
    {
      key: "servers",
      them: "Many servers, plus dedicated build servers",
      source: "https://coolify.io/docs/core/infrastructure/servers/overview",
    },
    {
      key: "license",
      them: "Free and open source (Apache 2.0)",
      source: "https://github.com/coollabsio/coolify",
    },
    {
      key: "maturity",
      them: "Releases since 2021, 65k GitHub stars",
      source: "https://github.com/coollabsio/coolify/releases",
    },
  ],
  different: [
    "No root SSH key in the app: a separate agent drives Docker.",
    "The firewall stays shut, port 22 included. The panel is on your tailnet.",
    "The admin account is set at install, not by the first visitor.",
  ],
  better: [
    "Many servers and build servers today, or a managed Cloud from $5/month.",
    "More than 360 one-click services, and backups for six database engines.",
    "Over five years of releases and more than 400 contributors.",
  ],
  pickThem: "You run several servers or a team, and want a proven tool today.",
  pickBrume: "You run one server, can try an alpha, and want no public panel or root SSH key.",
  checked: "2026-11-01",
};
