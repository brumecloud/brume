// Brume vs CapRover. Facts from archi/jalons/2026-11-01-recherche-vs/caprover.md.

import type { Competitor } from "./model";

export const caprover: Competitor = {
  slug: "caprover",
  name: "CapRover",
  kind: "self-hosted",
  icon: "caprover",
  seoTitle: "Brume vs CapRover: a CapRover alternative, no public panel",
  description:
    "CapRover is a proven Swarm PaaS, around since 2017. Brume is a young alpha with no public dashboard, no default password and Docker kept out of the panel.",
  oneLiner: "A proven, lightweight Swarm PaaS with a public dashboard.",
  verdict:
    "CapRover is a proven, lightweight Swarm PaaS with 360+ one-click apps. Brume is far younger and single-server, but starts with no public dashboard, no default password and no Docker socket in the panel.",
  rows: [
    {
      key: "panel",
      them: "Public, password captain42 until you change it",
      source: "https://caprover.com/docs/get-started.html",
    },
    {
      key: "docker",
      them: "Docker socket mounted in the captain service",
      source: "https://github.com/caprover/caprover/blob/master/src/utils/CaptainInstaller.ts",
    },
    {
      key: "hacked",
      them: "Broad control of the host",
      source: "https://caprover.com/docs-next/server/security",
    },
    {
      key: "ports",
      them: "80, 443, and 3000 until setup is done",
      source: "https://caprover.com/docs/firewall.html",
    },
    {
      key: "install",
      them: "Docker 25+, one docker run, then a CLI setup",
      source: "https://caprover.com/docs/get-started.html",
    },
    {
      key: "agents",
      them: "A CLI, no MCP server",
      source: "https://github.com/caprover",
    },
    {
      key: "backups",
      them: "Experimental config backup, no database dumps",
      source: "https://caprover.com/docs/backup-and-restore.html",
    },
    {
      key: "servers",
      them: "A Docker Swarm cluster, nodes added from the UI",
      source: "https://caprover.com/docs/app-scaling-and-cluster.html",
    },
    {
      key: "license",
      them: "Apache 2.0, plus an appendix for paid features",
      source: "https://github.com/caprover/caprover/blob/master/LICENSE",
    },
    {
      key: "maturity",
      them: "Releases since 2017, 15k GitHub stars",
      source: "https://github.com/caprover/caprover/releases",
    },
  ],
  different: [
    "No default password: the admin account is set at install.",
    "The dashboard lives on your tailnet, not on a public subdomain.",
    "Encrypted database dumps to your S3, and a full server restore.",
  ],
  better: [
    "A stable 1.x line, offered as production ready.",
    "Swarm clustering: add nodes from the UI, nginx balances the load.",
    "362 one-click apps, and it fits on a 1 GB instance.",
  ],
  pickThem: "You want a light, proven PaaS that can grow into a Swarm cluster.",
  pickBrume:
    "You want database backups and a private dashboard on one server, and can try an alpha.",
  checked: "2026-11-01",
};
