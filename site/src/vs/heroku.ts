// Brume vs Heroku, from archi/jalons/2026-11-01-recherche-vs/heroku.md.

import type { Competitor } from "./model";

export const heroku: Competitor = {
  slug: "heroku",
  name: "Heroku",
  kind: "managed",
  icon: "heroku",
  seoTitle: "Brume vs Heroku: a free, open source Heroku alternative",
  description:
    "Brume vs Heroku: per-dyno pricing and managed Postgres, or free git push deploys to your own VPS.",
  oneLiner: "The git push PaaS from 2007, owned by Salesforce.",
  verdict:
    "Heroku is a mature PaaS with managed Postgres and SOC and ISO certifications, in sustaining engineering since February 2026. Brume is a free alpha that deploys to a VPS you own.",
  rows: [
    {
      key: "runs",
      them: "Dynos in 2 regions, or 10 with Private Spaces",
      source: "https://devcenter.heroku.com/articles/regions",
    },
    {
      key: "data",
      them: "Heroku, on AWS; Postgres backups in S3",
      source:
        "https://devcenter.heroku.com/articles/heroku-postgres-data-safety-and-continuous-protection",
    },
    {
      key: "pricing",
      them: "From $5/month (Eco); Basic dynos $7/month each",
      source: "https://www.heroku.com/pricing",
    },
    {
      key: "lockin",
      them: "A Procfile at the repo root",
      source: "https://devcenter.heroku.com/articles/procfile",
    },
    {
      key: "databases",
      them: "Postgres from $5/month, Key-Value Store, Kafka",
      source: "https://www.heroku.com/pricing",
    },
    {
      key: "agents",
      them: "Local MCP server, in early development",
      source: "https://devcenter.heroku.com/articles/heroku-mcp-server",
    },
    {
      key: "scaling",
      them: "Autoscaling on Performance, Private and Shield dynos",
      source: "https://devcenter.heroku.com/articles/scaling",
    },
    {
      key: "compliance",
      them: "SOC 1/2/3, ISO 27001; PCI, HIPAA on Shield",
      source: "https://www.heroku.com/compliance",
    },
    {
      key: "license",
      them: "Closed platform; classic buildpacks are MIT",
      source: "https://github.com/heroku/heroku-buildpack-nodejs",
    },
    {
      key: "maturity",
      them: "Sustaining engineering since February 6, 2026",
      source: "https://www.heroku.com/blog/an-update-on-heroku/",
    },
  ],
  different: [
    "Free software on your own VPS, not a monthly price per dyno.",
    "Postgres, MongoDB and Redis on your server, backed up encrypted to your bucket.",
    "Any container, with persistent volumes for your databases.",
  ],
  better: [
    "Postgres with rollback up to a week, followers and HA, run for you.",
    "No server to run: OS patches, routing and TLS are Heroku's job.",
    "A known git push workflow, pipelines and over 200 add-ons.",
  ],
  pickThem: "You want git push with no server to run, managed Postgres, and PCI or HIPAA options.",
  pickBrume: "You want git push deploys to a VPS you own, for free, and an alpha is fine.",
  checked: "2026-11-01",
};
