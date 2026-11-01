# Brume vs CapRover — research notes

Checked: 2026-11-01. Official sources only (caprover.com docs and homepage, the GitHub repos caprover/caprover, caprover/caprover-website, caprover/caprover-cli and caprover/one-click-apps). "Derived" means our own inference from the cited facts, not a CapRover statement. Note: caprover.com/docs-next is CapRover's own preview of its next docs; it is used once below and labeled.

**What it is:** CapRover is a free, self-hosted PaaS ("Heroku on Steroids"). It is a Node.js/TypeScript app that drives Docker Swarm, nginx and Let's Encrypt behind a web dashboard and a CLI.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| License | Apache-2.0 plus an appendix that overrides it: paid features cannot be modified, no redistribution of a paid version, and changes to free features must stay free and open source. GitHub reports `NOASSERTION`. | https://github.com/caprover/caprover/blob/master/LICENSE | "In case of a conflict, the terms of this appendix supersede the general apache license." |
| Pricing, self-hosted | Free | https://caprover.com/ | "✓ Free forever ✓ No vendor lock-in ✓ Production ready" |
| Pricing, paid | CapRover Pro, a paid subscription adding 2FA (OTP), login alerts and build notifications. Pro price: not found (pro.caprover.com renders no pricing without JavaScript). | https://caprover.com/docs/troubleshooting-pro.html | "security upgrades such as login alerts and two factor authentication" |
| Stack | Node.js 24 (TypeScript, Express 5) | https://github.com/caprover/caprover/blob/master/release/dockerfile.release | `FROM node:24.22.1-alpine` |
| Install method | Install Docker 25+, then run `docker run -p 80:80 -p 443:443 -p 3000:3000 -e ACCEPTED_TERMS=true -v /var/run/docker.sock:/var/run/docker.sock -v /captain:/captain caprover/caprover`, then `caprover serversetup` from the CLI | https://caprover.com/docs/get-started.html | "docker run -p 80:80 -p 443:443 -p 3000:3000 -e ACCEPTED_TERMS=true" |
| Server requirements | Tested on Ubuntu 24.04 with Docker 25+. 512 MB RAM may be too little for builds; the docs point to 1 GB instances. AMD64 and ARM64 images. | https://caprover.com/docs/get-started.html | "CapRover is tested on Ubuntu 24.04 and Docker 25+." |
| Admin panel: default port | 3000 (plain HTTP) for initial setup, published in host mode. After setup the dashboard is served by nginx at `captain.<root-domain>` on 80/443. | https://caprover.com/docs/get-started.html | "Now you can access your CapRover from `captain.something.mydomain.com`." |
| Admin panel: public by default? | Yes. Port 3000 is published on all interfaces. **Derived:** afterwards the dashboard is a public vhost on 443. The docs say 3000 can be closed once a domain is attached. | https://caprover.com/docs/firewall.html | "`3000/tcp` for initial setup. You can close it after CapRover is attached to a domain." |
| Admin panel: default password | `captain42`, hard-coded fallback in source unless `DEFAULT_PASSWORD` is set. Password-only login (no username). | https://caprover.com/docs/get-started.html | "login to CapRover using the default password `captain42`" |
| Default password in code | Confirms the fallback | https://github.com/caprover/caprover/blob/master/src/user/Authenticator.ts | `const captainDefaultPassword = EnvVar.DEFAULT_PASSWORD ?? 'captain42'` |
| Admin panel: 2FA | Only with paid CapRover Pro (OTP) | https://caprover.com/docs/troubleshooting-pro.html | "This section is only applicable to CapRover Pro subscribers (paid plans)." |
| Admin panel: SSO | not found | — | — |
| How the panel reaches Docker | The host Docker socket is bind-mounted into the `captain-captain` Swarm service on the leader node (unless `CAPTAIN_DOCKER_API` is set) | https://github.com/caprover/caprover/blob/master/src/utils/CaptainInstaller.ts | `hostPath: CaptainConstants.dockerSocketPath,` |
| What an attacker who compromises the panel gets | **Derived:** the Docker socket on the Swarm leader is root-equivalent on that host, can schedule containers on every Swarm node, and exposes every app's env vars. CapRover's preview docs say so themselves. | https://caprover.com/docs-next/server/security | "A person with Docker socket access on the manager effectively has broad host control" |
| Inbound ports required by default | 80/tcp, 443/tcp, 443/udp, 3000/tcp. 996/tcp only for the built-in registry. 2377, 7946 and 4789 only between Swarm nodes. | https://caprover.com/docs/firewall.html | "Port `996/tcp` is used by CapRover's Docker registry." |
| Cloudflare Tunnel support | No official guide. The docs say proxies such as Cloudflare are not officially supported. A community `cloudflare-tunnel` one-click app exists. | https://caprover.com/docs/get-started.html | "CapRover does not officially support such use cases." |
| Multi-server | Yes, as a Docker Swarm cluster (UI or `docker swarm join`). Apps with persistent data stay on one node. A cluster needs a Docker registry. | https://caprover.com/docs/app-scaling-and-cluster.html | "It provides an option to use CapRover UI to set up a cluster of nodes." |
| AI agents: official CLI | Yes: `caprover` CLI (caprover/caprover-cli), latest v2.4.4 (2026-08-29). App tokens for CI. | https://github.com/caprover/caprover-cli | "Command Line Interface for CapRover." |
| AI agents: official MCP server | not found (no MCP repo in the caprover GitHub org, no mention in docs) | https://github.com/caprover | — |
| Database backups | Not found as a built-in feature. The built-in backup is a tar of CapRover's config, marked experimental. Images and volumes, including database data, need your own solution. No S3 target found. | https://caprover.com/docs/backup-and-restore.html | "Normal backup/restore works for everything except images and volumes." |
| Backup status | Experimental | https://caprover.com/docs/backup-and-restore.html | "Backup/Restore feature is still in experimental stage." |
| One-click templates/services | 362 community-maintained app definitions (`public/v4/apps/*.yml`, counted via GitHub API) | https://github.com/caprover/one-click-apps | "Community Maintained One Click Apps" |
| Maturity: first release | Repo created 2017-10-25. First GitHub release v0.1.2 on 2017-11-17. Latest v1.15.5 on 2026-10-23. 44 GitHub releases. | https://github.com/caprover/caprover/releases | not applicable (GitHub API data) |
| Maturity: GitHub stars | 15,195 | https://github.com/caprover/caprover | not applicable (GitHub API, 2026-11-01) |
| Maturity: contributors | 74 GitHub accounts (79 counting anonymous commit emails) | https://github.com/caprover/caprover/graphs/contributors | not applicable (GitHub API) |

## Where it is better than a young single-server alpha

1. **Track record:** 8 years of releases on a stable 1.x line. The homepage advertises it as production ready.
2. **Clustering:** Docker Swarm under the hood. You add nodes from the UI, and CapRover spreads stateless app instances and load-balances them with nginx.
3. **Catalogue:** 362 one-click apps (databases, WordPress, monitoring and more).
4. **Light footprint:** the docs treat a 1 GB RAM instance as the reference point (512 MB may be too little for builds).
5. **No lock-in:** apps keep running if CapRover is removed (README). The nginx template is fully customizable.

## Suggested verdict

Brume vs CapRover: CapRover is a proven, lightweight Swarm PaaS with a large app catalogue. Brume is much younger and single-server only, but it ships with no public dashboard, no default password and no Docker socket in the web app.
