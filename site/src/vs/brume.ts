// Brume's side of every comparison table, the same on every page. Each value describes alpha 0.1
// and comes from archi/base/ or docs/content/docs/.

import type { RowKey } from "./model";

export const brumeRows: Record<RowKey, { readonly label: string; readonly brume: string }> = {
  runs: { label: "Where it runs", brume: "Your own server, one VPS in alpha 0.1" },
  panel: { label: "Admin panel", brume: "On your tailnet only" },
  docker: { label: "Docker access", brume: "Only a separate agent, which listens on nothing" },
  hacked: { label: "If the panel is hacked", brume: "Nothing without your session" },
  ports: { label: "Inbound ports", brume: "None, through Cloudflare Tunnel" },
  install: { label: "Install", brume: "One command on Debian or Ubuntu" },
  data: { label: "Who holds your data", brume: "You: your server and your S3 bucket" },
  pricing: { label: "Price", brume: "Free. You pay for your VPS and your bucket" },
  lockin: {
    label: "Lock-in",
    brume: "Plain containers, no config file in your repo, a documented bucket format",
  },
  databases: { label: "Databases", brume: "PostgreSQL, MongoDB and Redis templates" },
  backups: { label: "Backups", brume: "Encrypted dumps on your S3, full restore of the server" },
  agents: {
    label: "AI agents",
    brume: "A CLI and a local MCP server. Irreversible actions wait for your approval",
  },
  servers: { label: "Several servers", brume: "One server in alpha 0.1, more planned" },
  scaling: { label: "Scaling and regions", brume: "One server, no autoscaling" },
  compliance: {
    label: "Compliance",
    brume: "No certifications: it runs on infrastructure you pick",
  },
  license: {
    label: "License",
    brume: "Free and open source (FSL-1.1-ALv2, Apache 2.0 after two years)",
  },
  maturity: { label: "Maturity", brume: "Alpha 0.1, in progress" },
};

/** The usual lines for a self-hosted platform: the same job, so the angle is security. */
export const selfHostedRows: readonly RowKey[] = [
  "panel",
  "docker",
  "hacked",
  "ports",
  "install",
  "agents",
  "backups",
  "servers",
  "license",
  "maturity",
];

/** The usual lines for a managed platform: your server, your data, a fixed cost. */
export const managedRows: readonly RowKey[] = [
  "runs",
  "data",
  "pricing",
  "lockin",
  "databases",
  "agents",
  "scaling",
  "compliance",
  "license",
  "maturity",
];
