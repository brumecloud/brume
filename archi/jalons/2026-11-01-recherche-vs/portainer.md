# Brume vs Portainer — research notes

Checked: 2026-11-01. Official sources only (docs.portainer.io, portainer.io, architecture.portainer.io, portainer.ai, the GitHub repos portainer/portainer, portainer/portainer-mcp and portainer/templates). "Derived" means our own inference from the cited facts, not a Portainer statement. Portainer is a general container-management UI rather than a git-push deploy platform, so some rows compare different things.

**What it is:** Portainer is a container-management UI for Docker, Swarm, Podman, Kubernetes and ACI environments. Community Edition (CE) is free under zlib. Business Edition (BE) is commercial and free for 3 nodes.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| License | CE: zlib (GitHub SPDX `Zlib`). BE: commercial license. | https://github.com/portainer/portainer | "Portainer is licensed under the zlib license." |
| CE intended use | CE is positioned as non-production: no support, no SLA | https://github.com/portainer/portainer | "Portainer CE is designed for homelabs, learning environments, personal projects, and other non-production deployments." |
| Pricing, self-hosted | CE free. BE free for 3 nodes ("Take 3", one license per organization). | https://github.com/portainer/portainer | "get 3 free nodes of Portainer Business for as long as you want them" |
| Pricing, paid | BE, priced per node: Starter $1,045/yr billed annually (5/10/15-node options), Scale $2,095/yr (5–25 nodes), Enterprise custom. No hosted/cloud SaaS found. | https://www.portainer.io/pricing | "Priced by the node, not the seat" |
| Stack | Go API with a TypeScript front end | https://github.com/portainer/portainer | not applicable (repo layout: `api/*.go`, `app/`) |
| Install method | `docker volume create portainer_data`, then `docker run -d -p 8000:8000 -p 9443:9443 --name portainer --restart=always -v /var/run/docker.sock:/var/run/docker.sock -v portainer_data:/data portainer/portainer-ce:lts` | https://docs.portainer.io/start/install-ce/server/docker/linux | "-v /var/run/docker.sock:/var/run/docker.sock -v portainer_data:/data portainer/portainer-ce:lts" |
| Server requirements | Latest Docker, sudo, Docker running as root, SELinux disabled (or `--privileged`). Tested on ARM64 and x86_64. RAM/CPU minimum not found. | https://docs.portainer.io/start/install-ce/server/docker/linux | "SELinux is disabled on the machine running Docker." |
| Admin panel: default port | 9443 (HTTPS). 9000 (HTTP) is optional and legacy. 8000 is the Edge tunnel server. | https://docs.portainer.io/start/install-ce/server/docker/linux | "By default, Portainer Server will expose the UI over port 9443" |
| Admin panel: public by default? | **Derived:** yes. The documented command publishes 9443 and 8000 on all host interfaces. Portainer's own guidance suggests a VPN before internet exposure. | https://www.portainer.io/how-to/how-to-correctly-secure-portainer-when-presented-on-the-internet | "exposing Portainer to the internet should be done carefully and with proper security controls in place." |
| Admin panel: auth options | Internal, LDAP, Active Directory, OAuth (LDAP and OAuth code ships in the CE repo). BE adds "pre-configured SSO provider templates". | https://docs.portainer.io/admin/settings/authentication | "Portainer provides its own internal authentication mechanism" |
| Admin panel: 2FA | No native MFA. MFA has to come from the OAuth identity provider. | https://architecture.portainer.io/ch4 | "Portainer has no native MFA capability" |
| How the panel reaches Docker | The host Docker socket is bind-mounted into the Portainer server container (local environment). Remote hosts use the Portainer Agent (inbound 9001) or the Edge Agent (outbound polling plus a tunnel to server port 8000). | https://docs.portainer.io/start/install-ce/server/docker/linux | "You are accessing Docker via Unix sockets. Alternatively, you can also connect via TCP." |
| What an attacker who compromises the panel gets | **Derived:** the mounted socket is root-equivalent on the host, and the server drives every connected environment (agents, Edge agents, Kubernetes). Portainer says so itself. | https://www.portainer.io/how-to/how-to-correctly-secure-portainer-when-presented-on-the-internet | "it has near root-level control over your infrastructure" |
| Inbound ports required by default | 9443 (UI/API). 8000 is also published by the documented command but only needed for Edge. 9001 on each standard-Agent host. | https://docs.portainer.io/start/install-ce/server/docker/linux | "The latter is optional and is only required if you plan to use the Edge compute features" |
| Agent port | Standard agents must be reachable on 9001 | https://docs.portainer.io/start/requirements-and-prerequisites | "TCP port 9001 (or 30778 for Kubernetes with NodePort) must be accessible on the Agent" |
| Cloudflare Tunnel support | Not found in official docs. Portainer's own Edge Agent removes inbound ports on managed hosts (not on the Portainer server). | https://docs.portainer.io/advanced/edge-agent | "There is now no need to expose the Portainer agents to the Internet." |
| Multi-server | Yes, its core use case: many environments (Docker standalone, Swarm, Podman, Kubernetes, ACI) through agents and Edge agents | https://github.com/portainer/portainer | "for Docker, Swarm, Podman, Kubernetes, and ACI environments" |
| AI agents: official CLI | No official management CLI found. The "CLI configuration options" docs page covers server startup flags. | https://docs.portainer.io/advanced/cli | — |
| AI agents: official MCP server | `portainer/portainer-mcp` (MIT, v2.45.3) exists but is no longer supported and may be archived. The supported route is Portainer Command, an MCP gateway in BE (free 3-node license included) aimed at Kubernetes fleets. | https://github.com/portainer/portainer-mcp | "Portainer MCP is no longer the supported way to use an LLM with Portainer." |
| Database backups | Not found for app databases. Portainer backs up only its own config. S3 as a target is BE-only. | https://docs.portainer.io/admin/settings/general | "It does not back up what you have deployed on your environments" |
| Portainer-config backup to S3 | BE only | https://docs.portainer.io/admin/settings/general | "This feature is only available in Portainer Business Edition." |
| One-click templates/services | 70 app templates in the official `templates.json` (v3 branch, counted) | https://github.com/portainer/templates | "This repository hosts the official templates ('Apps Templates') definitions for Portainer." |
| Maturity: first release | Repo created 2016-05-19. First GitHub release 1.0.0 on 2016-06-02. Latest 2.45.3 on 2026-10-21. 195 GitHub releases. | https://github.com/portainer/portainer/releases | not applicable (GitHub API data) |
| Maturity: GitHub stars | 38,757 | https://github.com/portainer/portainer | not applicable (GitHub API, 2026-11-01) |
| Maturity: contributors | 230 GitHub accounts (264 counting anonymous commit emails) | https://github.com/portainer/portainer/graphs/contributors | not applicable (GitHub API) |

## Where it is better than a young single-server alpha

1. **Scope:** it manages Docker, Swarm, Podman and Kubernetes environments, not just one server, from a single UI.
2. **Remote hosts without inbound ports:** the Edge Agent polls the server, so managed hosts need no open ports (the Portainer server itself still listens).
3. **Governance features:** BE adds RBAC, GitOps, edge management, audit and SSO (README), which matters once more than one person administers the hosts.
4. **Commercial backing:** a paid BE with support tiers and a free 3-node license. About 10 years of releases (since 2016) and 38k stars.
5. **Enterprise identity:** LDAP, Active Directory and OAuth/OIDC (Entra ID, Google, GitHub, Okta, Keycloak, Authentik).

## Suggested verdict

Brume vs Portainer: Portainer is a mature, general-purpose container manager for fleets and Kubernetes, not an opinionated app-deploy platform. Brume is a narrow single-server deploy tool whose default keeps the admin panel off the internet and away from the Docker socket.
