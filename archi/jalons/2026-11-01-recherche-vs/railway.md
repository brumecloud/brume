# Brume vs Railway — research notes

Checked: 2026-11-01. Official sources only (railway.com, docs.railway.com, blog.railway.com, github.com/railwayapp).

**What it is:** Railway is a managed, usage-billed cloud platform that builds and runs your services, databases and volumes on Railway's own infrastructure (Railway Metal) across 4 regions.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| Free trial | One-time $5 credit for 30 days, no card | https://railway.com/pricing | "Free Trial — $5 in credits for 30 days to try Railway. No credit card required." |
| Free plan | $0/month with $1 of usage credit per month; 1 vCPU / 0.5 GB per service, 1 replica | https://railway.com/pricing | "$0/month with $1 of monthly usage credits." |
| Hobby plan | $5/month, includes $5 of usage | https://railway.com/pricing | "Hobby — $5/month, including $5 of monthly usage credits." |
| Pro plan | $20/month per workspace, includes $20 of usage | https://railway.com/pricing | "Pro — $20/month per workspace, including $20 of monthly usage credits." |
| Seats | No per-seat fee on Pro; seats unlimited | https://railway.com/pricing | "Seats are unlimited and included — the fee is per workspace, not per seat." |
| Usage billing | Compute, memory, egress, storage billed per second on top of the plan fee | https://railway.com/pricing | "Resource consumption is billed in addition to the plan fee, per second." |
| CPU price | ~$20 per vCPU per month | https://railway.com/pricing | "CPU \| $0.00000772 per vCPU/s \| $20 per vCPU" |
| Memory price | ~$10 per GB per month | https://railway.com/pricing | "Memory \| $0.00000386 per GB/s \| $10 per GB" |
| Egress price | $0.05 per GB | https://railway.com/pricing | "Egress \| $0.05 per GB (services)" |
| Volume storage price | $0.15 per GB per month | https://docs.railway.com/reference/pricing/plans | "$0.15 / GB / month ($0.000003472222222 / GB / minute)" |
| Builds | Service builds are not billed | https://docs.railway.com/reference/pricing/plans | "Service builds are free." |
| Where apps run | Railway-owned hardware ("Railway Metal") in datacenters | https://docs.railway.com/railway-metal | "hardware that we own and operate in datacenters around the world." |
| Where apps run (caveat, June 2026) | Railway says it burst onto GCP and AWS during a demand spike; ~40% of workloads on cloud burst at the time | https://blog.railway.com/p/railway-metal-gen-2 | "We had to emergency provision cloud capacity meaning our hosts are multi-cloud in GCP/AWS/Metal." |
| Regions | 4: US West (California), US East (Virginia), EU West (Amsterdam), Southeast Asia (Singapore) | https://docs.railway.com/reference/regions | "Additional regions may be added in the future." |
| Who holds your data | Railway, in the region you pick; encrypted at rest | https://docs.railway.com/enterprise/security | "A service and its volumes run in the region you select when you create it." |
| Encryption at rest | AES-256 in every region | https://docs.railway.com/enterprise/security | "All data stored on Railway is encrypted at rest with AES-256 in every region." |
| Data export (Postgres) | pg_dump through a CLI tunnel or the public TCP proxy (proxy traffic billed as egress) | https://railway.com/guides/postgres-backups-restores | "This opens a local tunnel to the database and prints connection details" |
| Backups vs project deletion | Railway's own guide: only offsite logical dumps survive deleting the project | https://railway.com/guides/postgres-backups-restores | "They are also the only layer that survives deleting the project itself." |
| Volume backup download | not found (docs describe restore in-platform only) | https://docs.railway.com/volumes/backups | not found |
| Build: Dockerfile | Uses a root Dockerfile automatically | https://docs.railway.com/builds/dockerfiles | "Railway will look for and use a `Dockerfile` at the root of the source directory." |
| Build: Railpack | Zero-config builder (MIT, open source); Nixpacks (MIT) is in maintenance mode | https://docs.railway.com/builds/railpack | "Railpack automatically analyzes your code and turns it into a container image." |
| Nixpacks status | Maintenance mode, replaced by Railpack | https://github.com/railwayapp/nixpacks | "This project is currently in maintenance mode and is not under active development." |
| Proprietary config | railway.json / railway.toml ("Config as Code") deprecated; replaced by `.railway/railway.ts` IaC (Python, Go in beta) | https://docs.railway.com/infrastructure-as-code | "Existing Config as Code files stop being read on 2026-12-01 (hard cutoff)." |
| Portability | Dockerfile/Railpack produce standard images; platform config (IaC file, `*.railway.internal` DNS) is Railway-specific | https://docs.railway.com/networking/private-networking | "Services communicate over encrypted Wireguard tunnels using internal DNS." |
| Databases offered | PostgreSQL, MySQL, Redis, MongoDB (each with an HA variant); any other DB from a Docker image | https://docs.railway.com/databases | "Schema, queries, indexes and the data are always the user's." |
| Postgres HA | Patroni + etcd + HAProxy cluster with automatic failover | https://docs.railway.com/databases/postgresql-ha | "The cluster handles automatic leader election and failover." |
| Volume backups | Daily (kept 6 days), weekly (27 days), monthly (89 days); user turns them on | https://docs.railway.com/volumes/backups | "Backed up every 24 hours, kept for 6 days" |
| Postgres PITR | Available for single-node Postgres, must be enabled ahead of time | https://railway.com/guides/postgres-backups-restores | "PITR works for single-node Postgres" |
| Official CLI | Yes, open source (MIT) | https://docs.railway.com/cli | "The Railway CLI is open source." |
| Official MCP server | Yes: hosted at mcp.railway.com, plus local `railway mcp local`; old npm package deprecated into the CLI | https://docs.railway.com/ai/mcp-server | "The server runs at `mcp.railway.com`." |
| Agent-specific features | `railway-agent` tool / `railway agent` chat, a "use-railway" agent skill, cloud agents on persistent VMs | https://docs.railway.com/agents | "The use-railway skill teaches AI coding agents how to operate Railway." |
| Agent VM pricing | Sandboxes and cloud agents billed at separate VM rates ($50/vCPU, $50/GB RAM per month) | https://docs.railway.com/reference/pricing/plans | "are billed at VM rates instead of the container rates above." |
| SOC 2 | SOC 2 Type II and SOC 3 | https://docs.railway.com/enterprise/compliance | "Railway is SOC 2 Type II certified and SOC 3 certified." |
| HIPAA | BAA as a paid add-on above a monthly spend threshold | https://docs.railway.com/enterprise/compliance | "HIPAA BAA is an add-on with a paid monthly spend threshold." |
| Private networking | WireGuard-encrypted private network between services in an environment | https://docs.railway.com/enterprise/security | "traffic is over the private network, which is encrypted with WireGuard." |
| DDoS | L4-and-below mitigation on Metal, plus an "Under Attack Mode" | https://docs.railway.com/enterprise/security | "Railway Metal is built to mitigate attacks at network layer 4 and below." |
| Vertical scaling | Automatic | https://docs.railway.com/deployments/optimize-performance | "Vertical scaling is done automatically" |
| Horizontal scaling | Manual replica count (up to 42 replicas per service on Pro); no threshold autoscaler documented | https://docs.railway.com/deployments/optimize-performance | "Scale horizontally by manually increasing the number of replicas for a service." |
| Multi-region | Replicas per region; traffic routed to nearest region | https://docs.railway.com/deployments/optimize-performance | "Railway will automatically route public traffic to the nearest region" |
| Platform license | Platform source: not found (no open-source release located). Open-source parts: CLI (MIT), Railpack (MIT), Nixpacks (MIT) | https://github.com/railwayapp/cli | "The Railway CLI is open source." (docs.railway.com/cli) |
| Founded | not found (official founding year not stated). Infra history: started on one GCP region in 2020 | https://blog.railway.com/p/railway-metal-gen-2 | "a single GCP region in 2020, then 4 by 2023." |
| Funding | $100M Series B (Jan 2026) | https://railway.com/changelog/2026-01-23-100m-series-b | "We raised $100M!" |
| Scale numbers | 2M+ developers; tens of thousands of companies; 50M+ builds a month; 12,000+ new users a day | https://railway.com/changelog/2026-01-23-100m-series-b | "Over 2 million developers have shipped on Railway." |
| Scale numbers (2) | Builds and signups (Summer 2026) | https://blog.railway.com/p/railway-summer-update-2026 | "50M+ builds a month" / "12,000+ new users a day" |

## Where it is better than a self-hosted single-server alpha

1. **Managed, highly available databases.** One-click Postgres HA (Patroni + etcd + HAProxy, 2–7 replicas, automatic failover), PITR for single-node Postgres, and scheduled volume backups with daily/weekly/monthly retention. Brume runs one server, so it has no failover.
2. **Multi-region and replicas.** 4 regions, per-region replica counts, traffic sent to the nearest region, up to 42 replicas per service on Pro. A single VPS has one location and no horizontal scale-out.
3. **Compliance paperwork.** SOC 2 Type II and SOC 3 reports, a self-serve GDPR DPA, a HIPAA BAA add-on. A self-hosted alpha gives you none of these out of the box.
4. **Agent tooling that is further along.** A hosted MCP server at mcp.railway.com (OAuth), a local MCP mode, the `railway-agent` tool for multi-step debugging, an official agent skill, and cloud agents on persistent VMs.
5. **Mature team product.** 2M+ developers, unlimited seats on Pro, PR environments and a large template marketplace. Brume 0.1 has one admin.

## Suggested verdict

**Brume vs Railway:** Pick Railway if you want managed HA databases, several regions and SOC 2 paperwork, and you accept per-second usage billing on Railway's infrastructure. Pick Brume if you want a fixed VPS bill, your data and backups on your own server and S3 bucket, and a Dockerfile-based setup that doesn't depend on a vendor config format.
