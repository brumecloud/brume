// Brume vs Railway, from archi/jalons/2026-11-01-recherche-vs/railway.md.

import type { Competitor } from "./model";

export const railway: Competitor = {
  slug: "railway",
  name: "Railway",
  kind: "managed",
  icon: "railway",
  seoTitle: "Brume vs Railway: a Railway alternative on your own VPS",
  description:
    "Brume vs Railway: Railway bills usage per second on its cloud, in 4 regions. Brume runs your apps on your own VPS, with backups in your S3 bucket.",
  oneLiner: "Usage-billed cloud with managed databases, in 4 regions.",
  verdict:
    "Railway gives you highly available databases in 4 regions, billed by usage. Brume, an alpha, runs your apps on one VPS you own, at a fixed cost.",
  rows: [
    {
      key: "runs",
      them: "Railway's infrastructure, in 4 regions",
      source: "https://docs.railway.com/reference/regions",
    },
    {
      key: "data",
      them: "Railway, encrypted, in the region you pick",
      source: "https://docs.railway.com/enterprise/security",
    },
    {
      key: "pricing",
      them: "Free plan, or $5/month Hobby plus usage",
      source: "https://railway.com/pricing",
    },
    {
      key: "lockin",
      them: "Railway's own IaC format, .railway/railway.ts",
      source: "https://docs.railway.com/infrastructure-as-code",
    },
    {
      key: "databases",
      them: "PostgreSQL, MySQL, Redis, MongoDB, with HA variants",
      source: "https://docs.railway.com/databases",
    },
    {
      key: "agents",
      them: "CLI, plus a hosted or local MCP server",
      source: "https://docs.railway.com/ai/mcp-server",
    },
    {
      key: "scaling",
      them: "Replicas set by hand, vertical scaling automatic",
      source: "https://docs.railway.com/deployments/optimize-performance",
    },
    {
      key: "compliance",
      them: "SOC 2 Type II, SOC 3, HIPAA BAA add-on",
      source: "https://docs.railway.com/enterprise/compliance",
    },
    {
      key: "license",
      them: "Platform source not public; CLI is MIT",
      source: "https://github.com/railwayapp/cli",
    },
    {
      key: "maturity",
      them: "2M+ developers; $100M Series B in 2026",
      source: "https://railway.com/changelog/2026-01-23-100m-series-b",
    },
  ],
  different: [
    "Apps and databases run on your VPS, not on Railway's infrastructure.",
    "One fixed VPS bill, instead of a plan fee plus usage.",
    "Your database dumps go to your own bucket, outside the platform.",
  ],
  better: [
    "Managed Postgres HA with automatic failover, plus scheduled volume backups.",
    "4 regions, with public traffic routed to the nearest one.",
    "SOC 2 Type II and SOC 3 reports, and a HIPAA BAA.",
  ],
  pickThem:
    "You want HA databases in several regions and SOC 2 reports, and usage billing suits you.",
  pickBrume:
    "One server is enough and you want a fixed bill, with data and backups in your own bucket.",
  checked: "2026-11-01",
};
