// Brume vs Dokku, from archi/jalons/2026-11-01-recherche-vs/dokku.md.

import type { Competitor } from "./model";

export const dokku: Competitor = {
  slug: "dokku",
  name: "Dokku",
  kind: "self-hosted",
  icon: "dokku",
  seoTitle: "Brume vs Dokku: a Dokku alternative with a free web panel",
  description:
    "Dokku is a git push PaaS driven over SSH since 2013, with a paid web UI. Brume is an alpha with a free panel on your tailnet and zero open ports.",
  oneLiner: "A Heroku-style PaaS you drive with git push over SSH.",
  verdict:
    "Dokku is a mature, free PaaS you drive with git push and SSH. Brume is an alpha with a free panel that never touches Docker.",
  rows: [
    {
      key: "panel",
      them: "None in core. Dokku Pro adds one for $849",
      source: "https://pro.dokku.com/",
    },
    {
      key: "docker",
      them: "Its dokku user is in the docker group",
      source: "https://github.com/dokku/dokku/blob/master/debian/postinst",
    },
    {
      key: "hacked",
      them: "With an admin SSH key or Pro login: root-level access",
      source: "https://docs.docker.com/engine/install/linux-postinstall/",
    },
    {
      key: "ports",
      them: "SSH for git push, plus 80 and 443",
      source: "https://dokku.com/docs/deployment/application-deployment/",
    },
    {
      key: "agents",
      them: "Its CLI, over SSH. No official MCP server",
      source: "https://dokku.com/docs/deployment/remote-commands/",
    },
    {
      key: "backups",
      them: "Scheduled S3 backups through official plugins",
      source: "https://github.com/dokku/dokku-postgres",
    },
    {
      key: "servers",
      them: "Multi-node through its built-in K3s scheduler",
      source: "https://dokku.com/docs/deployment/schedulers/k3s/",
    },
    {
      key: "license",
      them: "Free, MIT",
      source: "https://github.com/dokku/dokku/blob/master/LICENSE",
    },
    {
      key: "maturity",
      them: "Since 2013, about 32k GitHub stars",
      source: "https://github.com/dokku/dokku",
    },
  ],
  different: [
    "A free panel, reachable on your tailnet only.",
    "No SSH or web port open: GitHub pushes arrive through Cloudflare Tunnel.",
    "Only an agent that listens on nothing touches Docker. The panel never does.",
  ],
  better: [
    "Public since 2013, with about 32k GitHub stars, and still released often.",
    "28 official plugins, from Postgres to RabbitMQ, with scheduled S3 backups.",
    "Heroku-style buildpacks and Procfiles, and a choice of five proxies.",
  ],
  pickThem:
    "You want a proven git push workflow over SSH, with official plugins for your databases.",
  pickBrume: "You want a web panel out of the box, with no SSH or web port open.",
  checked: "2026-11-01",
};
