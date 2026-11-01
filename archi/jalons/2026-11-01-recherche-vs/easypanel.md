# Brume vs Easypanel

Checked: 2026-11-01. Official sources only: easypanel.io (docs, pricing, changelog, blog, terms, its own comparison pages), the easypanel-io GitHub org, the get.easypanel.io install script, and the easypanel/easypanel image on Docker Hub. One non-easypanel.io page is flagged where it is used.

**What it is:** Easypanel is a closed-source, freemium web control panel. It installs on an Ubuntu server and turns it into a Docker Swarm + Traefik PaaS with one-click templates, managed databases and backups, priced per server.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| License | Proprietary. The core is closed source; template definitions are public on GitHub (the `easypanel-io/templates` repo has no license file). | https://easypanel.io/alternatives/dokku | "Core platform is closed source; template definitions are public on GitHub" |
| License (terms) | All rights reserved except those granted by license | https://easypanel.io/terms | "Easypanel and its licensors retain all rights in the Services, brand, website, and software" |
| Pricing | Free: $0, up to 3 projects. Hobby: $14.9 per server/month ($10.9 billed annually). Growth: $23.9 ($16.9). Business: $37.9 ($29.9). 30-day refund. | https://easypanel.io/pricing | "One license is a right to use Easypanel on one server (node, VM, or instance)." |
| Install method | `curl -sSL https://get.easypanel.io \| sh` as root. Alternatives: a `docker run ... easypanel/easypanel setup` command, or one-click images on DigitalOcean, AWS, Vultr, Linode, Hostinger and others. | https://easypanel.io/docs | "The script installs Docker when needed, initializes Docker Swarm, and starts Easypanel." |
| Server requirements | Fresh Ubuntu, at least 2 GB RAM, ports 80 and 443 free. The get.easypanel.io script also refuses to run on macOS or inside a container. | https://easypanel.io/docs | "Run this command as root on a fresh Ubuntu server with at least 2 GB of RAM." |
| Admin interface | Web UI (main interface). Also a REST API (Hobby+), an official CLI (Hobby+) and a remote MCP server (Hobby+). | https://easypanel.io/pricing | not applicable |
| Web UI default port | Not stated in the easypanel.io docs. The image declares `ExposedPorts 3000/tcp` (image config read from Docker Hub). A DigitalOcean Marketplace page (not easypanel.io) says "access Easypanel using the IP address of your server on port 3000." | https://hub.docker.com/r/easypanel/easypanel | `ExposedPorts: {"3000/tcp": {}}` (image config) |
| Web UI public by default? | Not found in the official docs. A panel domain can be set (API: "Update the panel domain"). | https://easypanel.io/docs/api | not found |
| Web UI auth: 2FA | Yes, on every plan including Free | https://easypanel.io/pricing | "Secure accounts with two-factor authentication." |
| Web UI auth: SSO | Not found (no SSO, OIDC or SAML in the pricing matrix or docs). Multiple users and project-level access control need Growth or higher. | https://easypanel.io/pricing | not found |
| How it reaches Docker | The documented setup and reset-password commands mount the host Docker socket (read-only). The panel then runs as the Swarm service `easypanel` next to `traefik`. How the running service mounts the socket is not documented. | https://easypanel.io/docs | "-v /var/run/docker.sock:/var/run/docker.sock:ro" |
| Runtime services | Swarm services `easypanel` and `traefik` | https://easypanel.io/docs/guides/uninstalling-easypanel | "docker service rm easypanel traefik" |
| Host-level actions from the panel | The API can reboot the host, prune Docker, deploy arbitrary images or Compose stacks, and so on. | https://easypanel.io/docs/api/server/reboot | "Immediately reboots the host server running Easypanel." |
| What an attacker gets by compromising the control interface | **Derived.** A panel admin session or admin API key can deploy any image or Compose stack, change Traefik routing, prune Docker and reboot the host. That is effectively root-equivalent control of the server through Docker, plus every stored secret, environment variable and backup credential. API keys (used by CLI and MCP) carry the full power of their user, and the MCP "connection URL" form puts the key in the URL path. | https://easypanel.io/docs/cli | "An API key acts as its Easypanel user and can modify or delete resources" |
| Inbound ports by default | 80 and 443 (Traefik; the installer exits if they are taken). The panel port is not documented (see the 3000 note above). The Easypanel docs do not cover SSH. | https://easypanel.io/docs | "Ports 80 and 443 must be available." |
| Cloudflare Tunnel support | Yes, built in, on every plan including Free, with per-service rules (cloudflared is updated by Easypanel). Whether the panel itself can be reached only through the tunnel: not found. | https://easypanel.io/pricing | "Expose services securely through Cloudflare Tunnel with per-service rules." |
| Multi-server support | Cluster support on Business only, still labelled alpha. Docker Swarm worker nodes with a single manager node, best for stateless services. Beta announced 2023-04-04. | https://easypanel.io/blog/multi-server-support-beta-release | "Easypanel currently does not support multiple manager nodes" |
| Cluster (pricing) | Business tier | https://easypanel.io/pricing | "Manage multiple servers as a cluster with worker nodes. This feature is in alpha." |
| AI agents: official CLI | Yes. The `easypanel` / `ep` CLI authenticates with a user API key and supports JSON output and multiple server profiles. Hobby+. Added in 2.33.0 (2026-07-31). | https://easypanel.io/docs/cli | "Manage Easypanel servers, projects, and services from your terminal." |
| AI agents: official MCP server | Yes. A remote Streamable HTTP endpoint at `https://<panel>/api/mcp` with 4 tools split by risk (query, mutation, destructive). Hobby+. Added in 2.33.0. | https://easypanel.io/docs/mcp | "Easypanel exposes four stable MCP tools rather than publishing every operation as a separate tool" |
| MCP / CLI release | Both shipped in 2.33.0 | https://easypanel.io/changelog/2-33-0 | "Added MCP support. Connection instructions are available next to each user's API key." |
| Database backups (to S3?) | Yes, on paid plans (Hobby+): scheduled DB backups with restore to S3-compatible storage (AWS, R2, B2, Wasabi, Spaces), Dropbox, Google Drive, FTP/SFTP or local. Volume backups too. Not on Free. | https://easypanel.io/pricing | "Scheduled database backups to S3-compatible storage, Dropbox, Google Drive, FTP/SFTP, or local storage with restore support." |
| One-click templates | 845 one-click installers. The homepage still says "750+" and the pricing tooltip says "790+". | https://easypanel.io/templates | "845 one-click installers" |
| Managed services | MySQL, MariaDB, Postgres, MongoDB, Redis, WordPress, Compose, Box | https://easypanel.io/pricing | "Managed MySQL, MariaDB, Postgres, MongoDB, and Redis with built-in admin tools." |
| Security audits (vendor claim) | Easypanel says it has had two independent professional audits. No public report was found. | https://easypanel.io/alternatives/dokku | "Two independent professional security audits" |
| Maturity: first release | 2022. The Docker Hub repo `easypanel/easypanel` was registered 2022-02-26, and the blog has a product changelog dated 2022-08-12. Latest is 2.38.0 (2026-10-22); the changelog lists 80 releases. | https://easypanel.io/blog/changelog-august-2022 | "Product updates August 12, 2022" |
| Maturity: GitHub stars | Not applicable: the core is not on GitHub. Public `easypanel-io/templates`: 208 stars. Docker Hub `easypanel/easypanel`: 7.7M pulls. Vendor claims: "66,000+ active servers" and "145,000+ self-hosters". (2026-11-01) | https://github.com/easypanel-io/templates | "Powering 66,000+ active servers" (easypanel.io homepage) |

## Where it is better than a young single-server alpha

1. **Breadth out of the box.** 845 one-click templates, plus managed MySQL, MariaDB, Postgres, MongoDB and Redis with admin tools, WordPress, Compose stacks, and Dockerfile, Nixpacks, Buildpacks or Railpack builds.
2. **Backups that already work.** Scheduled database and volume backups with restore, to S3-compatible storage (R2, B2, Wasabi, Spaces), SFTP, Dropbox or Google Drive (paid plans).
3. **Agent and automation tooling.** An official CLI with JSON output and CI support, and a remote MCP server whose tools are split by risk (read, mutate, destructive) so clients can gate destructive calls.
4. **Team and scale features.** Free 2FA, multiple users with project-level access control (Growth), and multi-node clusters (Business, alpha).
5. **Track record and support.** Public since 2022, 80 listed releases, 7.7M Docker Hub pulls, a company offering paid support, and a vendor-reported two independent security audits.

## Suggested verdict

**Brume vs Easypanel:** Easypanel is the more complete product today (templates, backups, CLI/MCP, teams, cluster alpha), sold as a closed-source per-server license. Brume trades that breadth for a smaller attack surface: no public ports, a panel reachable only on your tailnet, and a Docker socket the panel never touches.
