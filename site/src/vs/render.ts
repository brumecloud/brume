// Brume vs Render, from archi/jalons/2026-11-01-recherche-vs/render.md.

import type { Competitor } from "./model";

export const render: Competitor = {
  slug: "render",
  name: "Render",
  kind: "managed",
  icon: "render",
  seoTitle: "Brume vs Render: a Render alternative with a fixed bill",
  description:
    "Brume vs Render: autoscaling and managed Postgres on Render, or apps, databases and backups on one VPS you own, for the price of the VPS.",
  oneLiner: "PaaS on AWS and GCP, priced per instance.",
  verdict:
    "Render autoscales and runs Postgres with failover, at a price per instance. Brume, still in alpha, puts apps, databases and backups on one server you own.",
  rows: [
    {
      key: "runs",
      them: "Render's platform, in 5 regions",
      source: "https://render.com/docs/regions",
    },
    {
      key: "data",
      them: "Render, on AWS and GCP",
      source: "https://render.com/security",
    },
    {
      key: "pricing",
      them: "Free tier; instances from $7/month; Pro $25/month",
      source: "https://render.com/pricing",
    },
    {
      key: "lockin",
      them: "render.yaml Blueprints, Render's IaC format",
      source: "https://render.com/docs/infrastructure-as-code",
    },
    {
      key: "databases",
      them: "Managed Postgres and Key Value (Valkey)",
      source: "https://render.com/docs/key-value",
    },
    {
      key: "agents",
      them: "Hosted MCP server, open source",
      source: "https://render.com/docs/mcp-server",
    },
    {
      key: "scaling",
      them: "Autoscaling on Pro, up to 100 instances",
      source: "https://render.com/docs/scaling",
    },
    {
      key: "compliance",
      them: "SOC 2 Type 2, ISO 27001",
      source: "https://render.com/security",
    },
    {
      key: "license",
      them: "Platform source not public; MCP server is Apache-2.0",
      source: "https://github.com/render-oss/render-mcp-server",
    },
    {
      key: "maturity",
      them: "4.5M+ developers; Series C extension in 2026",
      source: "https://render.com/blog/series-c-extension",
    },
  ],
  different: [
    "Databases and their backups live on your server and your bucket.",
    "Brume is free: you pay for the VPS and bucket, not per instance.",
    "Your AI agent can do what the panel does. Irreversible actions wait for you.",
  ],
  better: [
    "Autoscaling on CPU or memory, up to 100 instances.",
    "Managed Postgres with point-in-time recovery and a standby for failover.",
    "A free tier for prototypes, and no server to run.",
  ],
  pickThem: "You need autoscaling, Postgres with point-in-time recovery, or ISO 27001 and HIPAA.",
  pickBrume: "Your load fits one VPS and you want apps, databases and backups on it, at its price.",
  checked: "2026-11-01",
};
