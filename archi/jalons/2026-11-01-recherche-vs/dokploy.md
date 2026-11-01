# Brume vs Dokploy — research notes

Checked: 2026-11-01. Official sources only (docs.dokploy.com, dokploy.com, the GitHub repos Dokploy/dokploy, Dokploy/website, Dokploy/mcp, Dokploy/cli and Dokploy/templates, and the install script). "Derived" means our own inference from the cited facts, not a Dokploy statement.

**What it is:** Dokploy is a self-hostable PaaS (a Next.js/TypeScript app running on Docker Swarm with Traefik) that deploys applications, Docker Compose stacks and databases. Its core is Apache-2.0, and its enterprise features are source-available. It is also sold as Dokploy Cloud.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| License | Mixed. Apache-2.0 for everything outside `/proprietary` directories. Code under `/proprietary` (SSO, audit logs, custom roles, whitelabeling, license keys) uses the Dokploy Source Available License (DSAL) 1.0, which needs a commercial agreement for production use. GitHub reports `NOASSERTION`. | https://github.com/Dokploy/dokploy/blob/canary/LICENSE.MD | "is available under the \"Apache License 2.0\" license" |
| License (proprietary part) | DSAL 1.0, applying only to `/proprietary` folders | https://github.com/Dokploy/dokploy/blob/canary/LICENSE_PROPRIETARY.md | "The Dokploy Source Available license (DSAL) version 1.0" |
| Pricing, self-hosted | Free | https://github.com/Dokploy/dokploy | "Dokploy is a free, self-hostable Platform as a Service (PaaS)" |
| Pricing, paid | Hobby $4.50/month per server (1 server, 1 org, 1 user). Startup from $15/month (3 servers, unlimited users), +$4.50 per extra server. Enterprise: contact sales (SSO/SAML, audit logs, fine-grained RBAC, SCIM, white labeling; cloud or self-hosted). 7-day free trial. The page does not say whether Hobby and Startup are Cloud-only. | https://dokploy.com/pricing | "per server (add as many servers as you'd like for $4.50/mo)" |
| Stack | Next.js 16 with Better Auth | https://github.com/Dokploy/dokploy/blob/canary/apps/dokploy/package.json | `"next": "16.4.1"` |
| Install method | `curl -sSL https://dokploy.com/install.sh \| sh`, run as root. It installs Docker, runs `docker swarm init`, creates an overlay network, then starts Postgres, the Dokploy service and a Traefik container. | https://dokploy.com/install.sh | "This script must be run as root" |
| Server requirements | At least 2 GB RAM and 30 GB disk. Tested on Ubuntu 18.04–24.04, Debian 10–12, Fedora 40 and CentOS 8/9. | https://docs.dokploy.com/docs/core/installation | "your server should have at least 2GB of RAM and 30GB of disk space." |
| Admin panel: default port | 3000 (plain HTTP), published in Swarm host mode | https://dokploy.com/install.sh | `--publish published=3000,target=3000,mode=host` |
| Admin panel: public by default? | Yes. The docs tell you to open 3000 in the firewall, then optionally remove the publish after a domain with HTTPS works. The docs also warn that Docker-published ports bypass UFW. | https://docs.dokploy.com/docs/core/installation | "Ensure that your server's firewall allows traffic on port 3000 to access the Dokploy web interface." |
| Docker bypasses UFW (Dokploy docs) | Ports published by Docker stay reachable even if UFW denies them | https://docs.dokploy.com/docs/core/remote-servers/security | "ports exposed by Docker containers remain accessible from the public internet even when UFW rules should block them" |
| Admin panel: 2FA | Yes: TOTP 2FA and passkeys (WebAuthn), configured per user under Settings → Profile | https://docs.dokploy.com/docs/core/account-security | "Protect your Dokploy account with passkeys (WebAuthn) and two-factor authentication." |
| Admin panel: SSO | OIDC and SAML (Auth0, Azure AD, Okta, Keycloak, Zitadel, any OIDC/SAML provider), plus SCIM. Enterprise only, and the code sits under `/proprietary` (DSAL). | https://docs.dokploy.com/docs/core/enterprise/sso | "Enterprise supports Single Sign-On via OpenID Connect (OIDC) and SAML." |
| How the panel reaches Docker | The host Docker socket is bind-mounted read-write into the `dokploy` Swarm service on the manager node. Traefik gets it read-only. | https://dokploy.com/install.sh | `--mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock` |
| How the panel reaches remote servers | SSH keys created in the Dokploy UI. The remote user is often root. | https://docs.dokploy.com/docs/core/remote-servers/instructions | "ensure you know the username (often `root`)" |
| What an attacker who compromises the panel gets | **Derived:** a read-write Docker socket on a Swarm manager is root-equivalent on the host (for example, run a container that mounts `/`) and controls services on every Swarm node. The attacker also gets every app's env vars and secrets, plus Dokploy-held SSH keys to remote servers, often as root. Docker's own guidance is quoted here. | https://docs.docker.com/engine/security/ | "only trusted users should be allowed to control your Docker daemon." |
| Inbound ports required by default | 80, 443 (TCP and UDP) and 3000. The install fails if any of them is in use. | https://docs.dokploy.com/docs/core/installation | "Port 3000: Dokploy web interface" |
| Cloudflare Tunnel support | Documented guide (cloudflared deployed as an app, routes to Traefik) and a one-click Cloudflared template. The guide does not route the dashboard itself through the tunnel. | https://docs.dokploy.com/docs/core/guides/cloudflare-tunnels | "an outbound-only connection to Cloudflare" |
| Tailscale (private panel) | A documented guide keeps the dashboard and Traefik reachable only over Tailscale. It is manual, and the docs recommend a provider firewall because of the UFW bypass. | https://docs.dokploy.com/docs/core/guides/tailscale | "Dokploy (port 3000) and Traefik (ports 80/443) are only accessible through Tailscale" |
| Multi-server | Yes: remote deployment servers and build servers over SSH, plus Docker Swarm cluster nodes. Monitoring of remote servers is not supported. | https://docs.dokploy.com/docs/core/remote-servers | "Remote servers allows you to deploy your apps remotely to different servers" |
| AI agents: official CLI | Yes: `@dokploy/cli` (Dokploy/cli, MIT), generated from the OpenAPI spec. Latest v0.30.7 (2026-09-18). | https://github.com/Dokploy/cli | "It provides 449 commands auto-generated from the Dokploy OpenAPI spec" |
| AI agents: official MCP server | Yes: `@dokploy/mcp` (Dokploy/mcp, Apache-2.0 LICENSE), stdio or HTTP, authenticated with a Dokploy API key. The panel also has a bring-your-own-key AI assistant for compose generation and log analysis. | https://github.com/Dokploy/mcp | "With 508 tools across 49 categories, this server provides complete coverage of the Dokploy API" |
| Database backups | Scheduled (cron) backups to an S3 destination, with a Test button. Documented destinations: AWS S3, Backblaze B2, Cloudflare R2 and Google Cloud Storage. | https://docs.dokploy.com/docs/core/databases/backups | "Select Destination S3 Bucket: Specify where your backups will be stored." |
| One-click templates/services | 593 template folders in the official templates repo (`blueprints/`, counted via GitHub API) | https://github.com/Dokploy/templates | "This is the official repository for the Dokploy Open Source Templates." |
| Maturity: first release | Repo created 2024-04-19. First GitHub release v0.0.1 on 2024-05-02. Latest v0.30.10 on 2026-10-22. 178 GitHub releases. | https://github.com/Dokploy/dokploy/releases | not applicable (GitHub API data) |
| Maturity: GitHub stars | 38,171 | https://github.com/Dokploy/dokploy | not applicable (GitHub API, 2026-11-01) |
| Maturity: contributors | 389 GitHub accounts (418 counting anonymous commit emails) | https://github.com/Dokploy/dokploy/graphs/contributors | not applicable (GitHub API) |

## Where it is better than a young single-server alpha

1. **Scale-out paths:** remote deployment servers, dedicated build servers and Docker Swarm clustering, all from one panel.
2. **Catalogue and Compose:** 593 templates in the official repo and native Docker Compose support.
3. **Backups:** scheduled database backups to S3-compatible destinations (AWS, B2, R2, GCS) with a one-click test.
4. **Identity:** passkeys and TOTP 2FA for everyone, plus paid SSO (OIDC/SAML), SCIM, RBAC and audit logs.
5. **Agent tooling and observability:** an official CLI (449 commands) and an official MCP server (508 tools), and log/metric shipping to Grafana, Datadog, New Relic and others (README). It has grown fast: 38k stars and about 390 contributors in about 2.5 years.

## Suggested verdict

Brume vs Dokploy: Dokploy already offers multi-server, 590+ templates, S3 backups and an official CLI and MCP server. Brume is a narrower single-server alpha whose default install keeps the panel off the internet and the Docker socket out of the web app. Dokploy can get partway there with its Tailscale guide, but its panel still holds the socket.
