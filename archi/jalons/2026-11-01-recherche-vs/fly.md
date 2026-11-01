# Brume vs Fly.io — research notes

Checked: 2026-11-01. Official sources only (fly.io, docs.fly.io, github.com/superfly).

**What it is:** Fly.io is a public cloud that runs your apps as Firecracker microVMs ("Fly Machines") on servers Fly owns, in 17 regions. Billing is pay-as-you-go, and it offers Managed Postgres, volumes and agent sandboxes ("Sprites").

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| Pricing model | Pay as you go, no monthly platform fee | https://docs.fly.io/about/pricing | "New organizations use Pay As You Go pricing." |
| Free tier | None for new organizations | https://docs.fly.io/about/pricing | "No. New organizations don't have a free tier or a monthly free usage allowance." |
| Free trial | Up to 2 hours of Machine runtime or 7 days | https://docs.fly.io/about/pricing | "free trial, which lasts for up to 2 hours of Machine runtime or 7 days, whichever comes first." |
| Compute price (smallest) | shared-cpu-1x, 256 MB: $2.19/month when running | https://docs.fly.io/about/pricing | "Started Machine, `shared-cpu-1x` with 256MB RAM \| $2.19/month" |
| Stopped Machines | Pay only for root filesystem | https://docs.fly.io/about/pricing | "$0.15/GB/month of rootfs" |
| Volumes | $0.15/GB/month provisioned; snapshots $0.08/GB/month, first 10 GB free | https://docs.fly.io/about/pricing | "First 10GB free each month." |
| Egress | $0.02/GB (NA, EU); $0.04/GB (APAC, Oceania, SA); $0.12/GB (Africa, India) | https://docs.fly.io/about/pricing | "$0.02/GB" (North America and Europe) |
| Dedicated IPv4 | $2/month | https://docs.fly.io/about/pricing | "$2/month" |
| Paid support | Standard $29/mo, Premium $199/mo, Enterprise from $2,500/mo | https://fly.io/pricing | "Support — Standard, $29 / mo" |
| Seats | not found (no per-seat price listed) | https://docs.fly.io/about/pricing | not found |
| Where apps run | Fly's own hardware in colocation datacenters (e.g. Equinix) | https://docs.fly.io/security/security-at-fly-io | "We run on our own hardware deployed in secure data centers like Equinix." |
| Regions | 17 (counted from the regions table): ams, arn, cdg, dfw, ewr, fra, gru, iad, jnb, lax, lhr, nrt, ord, sin, sjc, syd, yyz | https://docs.fly.io/reference/regions | "in datacenters around the world, on servers we run ourselves." |
| Who holds your data | Fly, on its own servers; volumes and DBs encrypted with LUKS | https://docs.fly.io/security/security-at-fly-io | "Customer information on databases and volumes at Fly.io is encrypted with the Linux LUKS block storage encryption secrets." |
| Volumes durability | Local NVMe on one physical server, not replicated; Fly recommends at least 2 volumes per app | https://docs.fly.io/volumes/overview | "Fly.io does not automatically replicate data among the volumes on an app" |
| Volume snapshots | Daily; kept 5 days by default (configurable 1–60 days) | https://docs.fly.io/volumes/overview | "We keep snapshots for five days by default" |
| Volume snapshot download | not found (docs only describe restoring into a new volume) | https://docs.fly.io/volumes/overview | not found |
| Data export (Postgres) | Standard pg_dump / pg_restore | https://docs.fly.io/mpg/ | "Use `pg_dump` and `pg_restore`" |
| Build options | Dockerfile, CNB buildpacks, or a prebuilt image | https://docs.fly.io/reference/configuration | "`dockerfile` accepts a relative path to a Dockerfile, or a URL." |
| Buildpacks | Supported via the `builder` setting | https://docs.fly.io/reference/configuration | "uses CNB Buildpacks and Builders to create the application image." |
| Proprietary config | `fly.toml` per app | https://docs.fly.io/reference/configuration | "You can configure an app for deployment on Fly.io using a `fly.toml` file." |
| Portability | OCI images/Dockerfiles are portable; `fly.toml` and 6PN/`.internal` networking are Fly-specific | https://docs.fly.io/networking/private-networking | "Fly Apps in an organization are connected by a mesh of WireGuard tunnels using IPv6 called a 6PN." |
| Databases offered | Managed Postgres (Fly); Upstash for Redis (partner); Tigris S3-compatible storage (partner, on Fly infra); volumes for self-run DBs | https://docs.fly.io/database-storage-guides | "Fly.io Managed Postgres is a production-ready Postgres service." |
| Managed Postgres pricing | From $38/month (Basic, shared-2x, 1 GB) up to $1,922/month | https://fly.io/pricing | "Every plan includes high availability, automated backups and connection pooling." |
| Managed Postgres HA | Primary + replica with automatic failover, within one region only | https://fly.io/mpg/ | "Every plan runs a primary and a replica with automatic failover." |
| Managed Postgres backups | Automatic, kept 10 days; point-in-time restore into a new cluster | https://fly.io/mpg/ | "automatically on a rolling schedule and keep those backups for ten days." |
| Official CLI | flyctl, open source (Apache-2.0) | https://github.com/superfly/flyctl | "Command line tools for fly.io services" (repo description) |
| Official MCP server | Yes, `fly mcp server` (local; SSE/stream options), marked experimental | https://docs.fly.io/flyctl/mcp-server/ | "[experimental] Start a flyctl MCP server" |
| MCP coverage | apps, certs, logs, machine, orgs, platform, secrets, status, volumes | https://docs.fly.io/model-context-protocol/flyctl-server | "most of the following commands and their subcommands are supported." |
| Agent-specific features | Sprites: persistent, checkpointing Linux sandboxes for coding agents; official agent plugins and SDKs; `fly mcp launch` to host MCP servers | https://fly.io/sprites/ | "Sprites are full Linux computers designed for agents instead of people." |
| Company focus | Repositioned around "computers for agents" (July 2026) | https://fly.io/news/fly-io-launches-computers-for-agents/ | "more than 8,000 of them are agent-native." |
| SOC 2 | SOC 2 Type 2 audited; hardware in ISO 27001 datacenters | https://fly.io/compliance | "Fly.io is SOC 2 Type 2 audited and our hardware runs in ISO 27001 datacenters." |
| HIPAA | $99/mo HIPAA package with pre-signed BAA, any paid plan | https://fly.io/pricing | "HIPAA package at $99/mo, with the BAA pre-signed" |
| Isolation | Firecracker microVMs | https://docs.fly.io/security/security-at-fly-io | "Compute jobs at Fly.io are virtualized using Firecracker" |
| Private networking | WireGuard IPv6 mesh (6PN) per organization; external hosts can join over WireGuard | https://docs.fly.io/networking/private-networking | "You can connect apps running outside of Fly.io to your 6PN using WireGuard." |
| DDoS | Relies on upstream providers; Fly says it is not a DDoS protection provider | https://docs.fly.io/about/healthcare | "Fly.io isn't a DDoS Protection provider." |
| DDoS (mitigation) | Upstream automated and manual mitigation | https://docs.fly.io/security/security-at-fly-io | "We work with upstream traffic providers to perform automated and manual DDoS mitigation." |
| Autoscaling (requests) | Fly Proxy autostop/autostart stops idle Machines and restarts them on request; does not create Machines | https://docs.fly.io/launch/autostop-autostart | "Fly Proxy autostop/autostart never creates or destroys Machines for you." |
| Autoscaling (metrics) | fly-autoscaler scales Machine count on any metric (e.g. queue depth) | https://docs.fly.io/launch/autoscale-by-metric | "The metrics-based autoscaler scales an app's Machines based on any metric, such as pending work items or queue depth." |
| Horizontal / multi-region | `fly scale count` across listed regions | https://docs.fly.io/launch/scale-count | "creates and/or destroys Machines to reach the specified target count across the regions you list" |
| Platform license | Platform source: not found (no open-source release located). Open-source parts: flyctl, LiteFS, Corrosion, macaroon (Apache-2.0) | https://github.com/superfly | not found (no statement on platform license) |
| Founded | Building on own hardware since 2017 | https://fly.io/about/ | "We've been building a developer-focused public cloud on our own hardware since 2017." |
| Funding / leadership | $25M Series D, new CEO Scott Johnston (July 24, 2026) | https://fly.io/news/fly-io-launches-computers-for-agents/ | "$25 million in Series D funding." |
| Scale numbers | 37,000+ customers; hundreds of thousands of developers | https://fly.io/news/fly-io-launches-computers-for-agents/ | "More than 37,000 customers build on Fly.io." |

## Where it is better than a self-hosted single-server alpha

1. **Global footprint on owned hardware.** 17 regions, and `fly scale count` spreads Machines across the regions you list. A single VPS sits in one place.
2. **Scale-to-zero and elastic Machines.** Autostop/autostart stops idle Machines and wakes them on the next request; fly-autoscaler scales on queue depth or any metric; a running 256 MB Machine costs $2.19/month.
3. **Managed Postgres with HA and PITR.** Every plan has a primary and a replica with automatic failover, 10 days of automatic backups and point-in-time restore into a new cluster.
4. **Isolation and compliance.** Firecracker microVM isolation, SOC 2 Type 2, ISO 27001 datacenters, and a $99/mo HIPAA package with a pre-signed BAA.
5. **Agent infrastructure.** Sprites (persistent, checkpointing sandboxes with official plugins for coding agents) and `fly mcp launch` for hosting MCP servers. Agent-native companies are now Fly's stated focus (8,000+ customers).

## Suggested verdict

**Brume vs Fly.io:** Fly.io is the better fit if you need apps close to users in many regions, scale-to-zero microVMs or agent sandboxes, and you're fine with usage billing and Fly-specific `fly.toml`/6PN plumbing. Brume is the better fit if one server in one place is enough and you want a fixed bill, data and backups on your own S3 bucket, and no inbound ports.
