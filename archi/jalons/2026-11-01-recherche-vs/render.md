# Brume vs Render — research notes

Checked: 2026-11-01. Official sources only (render.com, render.com/docs, render.com/blog, github.com/render-oss).

**What it is:** Render is a managed cloud platform (PaaS) that runs web services, workers, cron jobs, static sites, managed Postgres and Key Value on AWS and GCP. You pay a flat workspace fee plus fixed-price instance sizes.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| Workspace plans | Hobby $0 + compute; Pro $25/mo + compute; Scale $499/mo + compute; Enterprise custom (since April 23, 2026) | https://render.com/pricing | "Pro … For teams deploying production-grade apps and agents. $ 25 /mo + compute" |
| Free tier | Free web services, Postgres and Key Value instances; 750 free instance hours per workspace per month | https://render.com/docs/free | "Render grants 750 Free instance hours to each workspace per calendar month." |
| Free tier limits | Free web services spin down after 15 min idle; free Postgres expires after 30 days; not for production | https://render.com/docs/free | "Free Render Postgres databases expire 30 days after creation." |
| Free static sites | Static sites are free | https://render.com/pricing | "Deploy websites for free, forever" |
| Compute pricing | Fixed instance sizes; smallest paid web service: Starter, $7/month, 512 MB RAM, 0.5 CPU | https://render.com/pricing | "$7/month 512 MB RAM 0.5c-512mb" |
| Seats | Hobby: 1 seat. Pro and above: unlimited seats, no per-seat fee | https://render.com/pricing | "Unlimited seats & services" |
| Egress | Included: Hobby 5 GB, Pro 25 GB, Scale 1 TB; then $0.15/GB | https://render.com/docs/outbound-bandwidth | "$0.15 per GB" |
| Disk storage | Persistent disks $0.25/GB/month; Postgres storage $0.30/GB | https://render.com/pricing | "Persistent disks $0.25 per GB per month" |
| Where apps run | AWS and GCP (listed as hosting subprocessors); Render runs its own Kubernetes on AWS | https://render.com/security | AWS: "Hosting / Cloud Platform"; GCP: "Hosting / Cloud Platform" |
| Where apps run (detail) | Self-managed Kubernetes on AWS EC2/S3/ELB | https://render.com/blog/how-render-services-stayed-up-during-the-aws-october-outage | "Render operates its own Kubernetes clusters on AWS instead of using EKS" |
| Regions | 5: Oregon, Ohio, Virginia, Frankfurt, Singapore | https://render.com/docs/regions | not found (region count not stated in prose; 5 counted from the list) |
| Which cloud per region | not found (not documented officially) | https://render.com/docs/regions | not found |
| Who holds your data | Render, on AWS/GCP | https://render.com/security | AWS / GCP: "Hosting / Cloud Platform" |
| Data export (Postgres) | Manual logical backups you can download; kept 7 days; paid instances only | https://render.com/docs/postgresql-backups | "Click any export's download link to save it to your local machine." |
| Data export (free Postgres) | No backups; run pg_dump yourself | https://render.com/docs/postgresql-backups | "Render does not create logical backups for databases on the Free compute plan." |
| Data export (disks) | Copy files out over SCP or Magic-Wormhole; snapshot download not found | https://render.com/docs/disks | "enables you to transfer files to and from your disk-backed service without using SSH and SCP." |
| Build: native runtimes | Node/Bun, Python, Ruby, Go, Rust, Elixir | https://render.com/docs/native-runtimes | "JavaScript / TypeScript (supports both Node.js and Bun)" |
| Build: Dockerfile | Docker runtime builds from a Dockerfile | https://render.com/docs/web-services | "also provides a Docker runtime for building and running a custom image from a `Dockerfile`." |
| Buildpacks / Nixpacks | not found (no mention in the docs pages checked) | https://render.com/docs/native-runtimes | not found |
| Proprietary config | `render.yaml` Blueprint (Render's IaC format); also an official Terraform provider (Apache-2.0) | https://render.com/docs/infrastructure-as-code | "Blueprints are Render's infrastructure-as-code (IaC) model for defining, deploying, and managing multiple resources" |
| Portability | Dockerfile apps are portable; the Blueprint spec has no stated portability or open-spec status | https://render.com/docs/infrastructure-as-code | not found (no portability statement) |
| Databases offered | Render Postgres (managed) and Render Key Value (Valkey 8, Redis-compatible) | https://render.com/docs/key-value | "Key Value instances are compatible with virtually all clients that interact with Redis®*." |
| Postgres pricing | Free ($0, 30-day limit); Basic-256mb $6/month; up to 128 CPU / 1024 GB RAM | https://render.com/pricing | "Basic-256mb" "$6/month" |
| Postgres PITR | Paid databases; 3-day window on Hobby, 7 days on Pro and above | https://render.com/docs/postgresql-backups | "Render continually backs up paid Render Postgres databases." |
| Postgres HA | Standby in a separate zone, automatic failover after 30 s; needs ≥1 CPU plan, PG 13+ | https://render.com/docs/postgresql-high-availability | "Render automatically triggers a failover to your database's standby instance." |
| Disk backups | Daily snapshot, kept 7 days | https://render.com/docs/disks | "Render automatically creates a snapshot of your persistent disk once every 24 hours." |
| Key Value persistence | Paid only; none on free | https://render.com/docs/key-value | "Data persistence is not available for free Key Value instances." |
| Official CLI | Yes, public repo (render-oss/cli, Apache-2.0); interactive or non-interactive | https://render.com/docs/cli | "The CLI also supports non-interactive use in scripts and CI/CD." |
| Official MCP server | Yes: hosted at https://mcp.render.com/mcp; open source (render-oss/render-mcp-server, Apache-2.0) | https://render.com/docs/mcp-server | "enables you to manage your Render infrastructure directly from compatible AI apps" |
| MCP limits | Can create services/DBs, read-only SQL, logs, metrics, deploys; no deletions or scaling changes | https://render.com/docs/mcp-server | "The MCP server does not support modifying scaling settings or other operational service controls." |
| Agent-specific features | Official agent skills, plugins for Cursor, Codex, Claude Code; llms.txt docs; Sandboxes and Workflows products | https://render.com/docs/llm-support | "Render's official skills enable agents to deploy, debug, and monitor your apps." |
| SOC 2 / ISO | SOC 2 Type 2 and ISO 27001 | https://render.com/security | "SOC 2 Type 2" — "Trust services criteria for security, confidentially, and availability." |
| HIPAA | HIPAA-compliant workspaces on Scale; BAA; 20% compute premium on enabled workspaces | https://render.com/pricing | "HIPAA BAA" … "20% compute premium for enabled workspaces" |
| Private networking | Automatic private network per region and workspace; AWS PrivateLink on Pro | https://render.com/docs/private-network | "Your Render services in the same region can communicate over their shared private network" |
| DDoS | Free Cloudflare-backed DDoS protection on all services | https://render.com/docs/ddos-protection | "Render provides free distributed denial-of-service protection to every application and website hosted on our platform." |
| Autoscaling | CPU/memory-target horizontal autoscaling; Pro plan or higher | https://render.com/docs/scaling | "Autoscaling requires a Pro workspace plan or higher." |
| Horizontal scaling limits | Up to 100 instances (smaller plans); a service with a disk cannot scale out | https://render.com/docs/scaling | "Services with an attached persistent disk cannot scale to multiple instances." |
| Regions (scaling) | Services and private network are per region; no multi-region single service found | https://render.com/docs/private-network | "Other Render services are on the same private network if they're deployed in the same region" |
| Platform license | Platform source: not found (no open-source release located). Open-source parts: CLI, MCP server, Terraform provider (Apache-2.0), skills (MIT) | https://github.com/render-oss/render-mcp-server | "To explore the implementation of the MCP server itself, see the open-source project" (render.com/docs/mcp-server) |
| Founded | Exact founding year not found; public launch at TechCrunch Disrupt 2019 | https://render.com/blog/render-series-b | "We've grown quite a bit since launching at TechCrunch Disrupt in 2019." |
| Funding | $258M raised total; $100M Series C extension at $1.5B (Feb 17, 2026) | https://render.com/about | "We have raised $258M from world-class investors." |
| Scale numbers | 4.5M+ developers; 250,000+ new developers per month | https://render.com/blog/series-c-extension | "4.5 million+ developers" / "more than 250,000 new developers joining every month." |

## Where it is better than a self-hosted single-server alpha

1. **Load-based autoscaling.** Horizontal autoscaling on CPU and/or memory targets (Pro and above), up to 100 instances per service. A single VPS cannot add machines.
2. **Managed Postgres with recovery and HA.** Continuous backups with PITR (3 days on Hobby, 7 days on Pro and above), downloadable logical exports, a standby in a separate zone with automatic failover, and plans up to 128 CPU / 1 TB RAM.
3. **Compliance and edge security.** SOC 2 Type 2, ISO 27001, HIPAA workspaces with a BAA, a GDPR DPA, and free Cloudflare-backed DDoS protection on every service.
4. **A free starting point.** Free static sites, plus 750 free instance hours, free Postgres (30 days) and free Key Value for prototypes. You need no server of your own.
5. **Company scale and agent tooling.** 4.5M+ developers and $258M raised. A hosted, open-source MCP server, official agent skills and plugins, and Render Sandboxes and Workflows for agent workloads.

## Suggested verdict

**Brume vs Render:** Render is the better fit if you want autoscaling, managed Postgres with PITR and HA, and SOC 2 / ISO 27001 / HIPAA paperwork, and you accept per-instance prices and bandwidth overages on AWS/GCP. Brume is the better fit if you want one fixed VPS bill, your data and backups on your own server and S3 bucket, and no Render-specific `render.yaml` to maintain.
