// Brume vs Kamal, from archi/jalons/2026-11-01-recherche-vs/kamal.md.

import type { Competitor } from "./model";

export const kamal: Competitor = {
  slug: "kamal",
  name: "Kamal",
  kind: "self-hosted",
  icon: "kamal",
  seoTitle: "Brume vs Kamal: a Kamal alternative with zero open ports",
  description:
    "Kamal deploys containers over SSH from your machine, with no panel. Brume, an alpha, deploys on git push, with a tailnet panel and no open ports.",
  oneLiner: "A CLI from 37signals that deploys containers over SSH.",
  verdict:
    "Kamal is a proven deploy CLI that leaves only a small proxy on your servers. Brume, an alpha, adds a panel and deploys on git push, with no open ports.",
  rows: [
    {
      key: "panel",
      them: "None: a CLI over SSH",
      source: "https://kamal-deploy.org/docs/commands/view-all-commands/",
    },
    {
      key: "docker",
      them: "Docker commands over SSH, as root by default",
      source: "https://kamal-deploy.org/docs/configuration/ssh/",
    },
    {
      key: "ports",
      them: "SSH, plus 80 and 443 for kamal-proxy",
      source: "https://kamal-deploy.org/docs/configuration/proxy/",
    },
    {
      key: "install",
      them: "A Ruby gem locally. Servers need only SSH",
      source: "https://kamal-deploy.org/docs/installation/",
    },
    {
      key: "agents",
      them: "The CLI itself. No official MCP server",
      source: "https://kamal-deploy.org/docs/commands/view-all-commands/",
    },
    {
      key: "backups",
      them: "None built in. Databases run as accessories",
      source: "https://kamal-deploy.org/docs/configuration/accessories/",
    },
    {
      key: "servers",
      them: "Many, with roles. You add the load balancer",
      source: "https://kamal-deploy.org/docs/installation/",
    },
    { key: "license", them: "Free, MIT", source: "https://github.com/basecamp/kamal" },
    {
      key: "maturity",
      them: "Deploys HEY and other 37signals apps",
      source: "https://kamal-deploy.org/",
    },
  ],
  different: [
    "A panel on your tailnet, and a deploy on every git push.",
    "No inbound ports, SSH included: public traffic comes through Cloudflare Tunnel.",
    "Encrypted database dumps on your S3, and a full restore of the server.",
  ],
  better: [
    "Proven: 37signals deploys HEY with it, and it has been public since 2023.",
    "Deploys to many servers from one file, with roles like web and workers.",
    "No control plane to attack: only kamal-proxy runs, without the Docker socket.",
  ],
  pickThem: "You deploy to several servers from a terminal or CI, and want no panel at all.",
  pickBrume: "You run one server and want a panel, deploys on git push and no open ports.",
  checked: "2026-11-01",
};
