// Brume vs Vercel, from archi/jalons/2026-11-01-recherche-vs/vercel.md.

import type { Competitor } from "./model";

export const vercel: Competitor = {
  slug: "vercel",
  name: "Vercel",
  kind: "managed",
  icon: "vercel",
  seoTitle: "Brume vs Vercel: a Vercel alternative, databases included",
  description:
    "Brume vs Vercel: Functions and beta containers on Vercel's global network, or containers and databases on your own server, at a fixed cost.",
  oneLiner: "Frontend cloud, Next.js first, running Functions on AWS.",
  verdict:
    "Vercel runs Functions, and containers in beta, on a global network, but hosts no database. Brume, an alpha, runs apps and databases on one server you own.",
  rows: [
    {
      key: "runs",
      them: "Vercel's network: 19 compute regions, 126 PoPs",
      source: "https://vercel.com/docs/regions",
    },
    {
      key: "data",
      them: "Vercel, mostly on AWS; may go to the US",
      source: "https://vercel.com/docs/security/compliance",
    },
    {
      key: "pricing",
      them: "$20/month per deploying seat, plus usage",
      source: "https://vercel.com/docs/plans/pro-plan",
    },
    {
      key: "lockin",
      them: "vercel.json for builds, routes, crons, regions",
      source: "https://vercel.com/docs/project-configuration",
    },
    {
      key: "databases",
      them: "None of its own; Marketplace partners like Neon",
      source: "https://vercel.com/docs/postgres",
    },
    {
      key: "agents",
      them: "Hosted MCP server with OAuth, on all plans",
      source: "https://vercel.com/docs/agent-resources/vercel-mcp",
    },
    {
      key: "scaling",
      them: "Functions scale to 30,000 concurrency, and to zero",
      source: "https://vercel.com/docs/functions/limitations",
    },
    {
      key: "compliance",
      them: "SOC 2 Type 2, ISO 27001, PCI DSS, HIPAA",
      source: "https://vercel.com/docs/security/compliance",
    },
    {
      key: "license",
      them: "Closed platform; Next.js is MIT",
      source: "https://github.com/vercel/next.js",
    },
    {
      key: "maturity",
      them: "Since 2016, first as ZEIT",
      source: "https://vercel.com/blog/zeit-is-now-vercel",
    },
  ],
  different: [
    "Containers run as plain containers, not as Functions with time limits.",
    "PostgreSQL, MongoDB and Redis run on your server; Vercel hosts no database.",
    "Your data and backups stay on your server and your S3 bucket.",
  ],
  better: [
    "126 PoPs and 19 compute regions put your app near users.",
    "Functions scale up to 30,000 concurrency, and down to zero.",
    "Preview URL per pull request, plus ISR, image optimization, Skew Protection.",
  ],
  pickThem: "You ship a Next.js or frontend app that needs a global CDN and scale-to-zero.",
  pickBrume: "Your app needs a database or a long-running container, on a server you own.",
  checked: "2026-11-01",
};
