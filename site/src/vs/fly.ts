// Brume vs Fly.io, from archi/jalons/2026-11-01-recherche-vs/fly.md.

import type { Competitor } from "./model";

export const fly: Competitor = {
  slug: "fly",
  name: "Fly.io",
  kind: "managed",
  icon: "fly",
  seoTitle: "Brume vs Fly.io: a self-hosted Fly.io alternative",
  description:
    "Brume vs Fly.io: pay-as-you-go Machines in 17 regions, or one VPS you own with zero open ports and a fixed bill. What each one does better.",
  oneLiner: "MicroVMs on Fly's own servers, in 17 regions.",
  verdict:
    "Fly.io runs Machines close to users in 17 regions, and stops them when idle. Brume is an alpha for one VPS in one place, with a fixed bill and zero open ports.",
  rows: [
    {
      key: "runs",
      them: "Fly's own servers, in 17 regions",
      source: "https://docs.fly.io/reference/regions",
    },
    {
      key: "data",
      them: "Fly, encrypted with LUKS",
      source: "https://docs.fly.io/security/security-at-fly-io",
    },
    {
      key: "pricing",
      them: "Pay as you go; 256 MB Machine $2.19/month",
      source: "https://docs.fly.io/about/pricing",
    },
    {
      key: "lockin",
      them: "A fly.toml config per app",
      source: "https://docs.fly.io/reference/configuration",
    },
    {
      key: "databases",
      them: "Managed Postgres; Redis and S3 via partners",
      source: "https://docs.fly.io/database-storage-guides",
    },
    {
      key: "agents",
      them: "Experimental MCP server in flyctl",
      source: "https://docs.fly.io/flyctl/mcp-server/",
    },
    {
      key: "scaling",
      them: "Autoscaling on any metric, like queue depth",
      source: "https://docs.fly.io/launch/autoscale-by-metric",
    },
    {
      key: "compliance",
      them: "SOC 2 Type 2, ISO 27001 datacenters",
      source: "https://fly.io/compliance",
    },
    {
      key: "license",
      them: "Platform source not public; flyctl is Apache-2.0",
      source: "https://github.com/superfly/flyctl",
    },
    {
      key: "maturity",
      them: "37,000+ customers; Series D in 2026",
      source: "https://fly.io/news/fly-io-launches-computers-for-agents/",
    },
  ],
  different: [
    "Zero open ports: traffic arrives through a Cloudflare Tunnel.",
    "Encrypted backups go to your S3 bucket and restore the whole server.",
    "One VPS bill, not per-Machine, volume and egress charges.",
  ],
  better: [
    "17 regions, with Machines spread across the ones you list.",
    "Idle Machines stop, then wake on the next request.",
    "Managed Postgres with a replica, failover and 10 days of backups.",
  ],
  pickThem: "Your users are worldwide, or your Machines should sleep when nobody calls them.",
  pickBrume: "One location serves your users and you want a fixed bill and zero open ports.",
  checked: "2026-11-01",
};
