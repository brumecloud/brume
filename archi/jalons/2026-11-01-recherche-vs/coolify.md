# Brume vs Coolify — research notes

Checked: 2026-11-01. Official sources only (coolify.io docs, pricing, GitHub repo, install script, compose files). "Derived" means our own inference from the cited facts, not a Coolify statement.

**What it is:** Coolify is an open-source, self-hostable PaaS (a Laravel/PHP app) that deploys applications, databases and one-click services to your own servers over SSH, as an alternative to Heroku, Netlify and Vercel.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| License | Apache-2.0 (GitHub SPDX `Apache-2.0`) | https://github.com/coollabsio/coolify | "Open source & free forever" |
| Pricing, self-hosted | Free | https://coolify.io/pricing/ | "Free forever." |
| Pricing, paid | Coolify Cloud (managed control plane, you bring servers): $5/month base for 2 servers, +$3/month per extra server; yearly billing saves 20%. No other paid tier listed. | https://coolify.io/pricing/ | "$5 /month base price (connect 2 servers)" / "+ $3 /month per additional server" |
| Stack | Laravel 12 on PHP 8.4 | https://github.com/coollabsio/coolify/blob/main/composer.json | `"laravel/framework": "^12.70.1"` |
| Install method | One-line script run as root: `curl -fsSL https://cdn.coollabs.io/coolify/install.sh \| bash`. It installs Docker and OpenSSH if missing and runs Coolify with Docker Compose from `/data/coolify/source`. A manual method for non-root users exists. | https://coolify.io/docs/get-started/installation | "Requires root user access." |
| Server requirements | 2 CPU cores, 2 GB RAM, 10 GB disk; Linux only; amd64 or arm64 | https://coolify.io/docs/get-started/installation | "Coolify needs 2 CPU cores, 2 GB RAM, and 10 GB disk space to run." |
| Admin panel: default port | 8000 (plain HTTP). The `coolify` container also publishes 6001 (realtime) and 6002 (web terminal). | https://cdn.coollabs.io/coolify/docker-compose.prod.yml | `"${APP_PORT:-8000}:8080"` |
| Admin panel: public by default? | Yes. Ports 8000/6001/6002 are published on the host, and the installer prints `http://<public-ip>:8000`. The docs say to close them once a domain works. | https://coolify.io/docs/knowledge-base/server/firewall | "you can close public access to ports 8000, 6001, and 6002." |
| Admin panel: first-run exposure | The first visitor creates the admin account. Env vars (`ROOT_USERNAME`, `ROOT_USER_EMAIL`, `ROOT_USER_PASSWORD`) can pre-create it. | https://coolify.io/docs/get-started/installation | "Anyone who reaches the registration page first can become the instance admin and gain root access" |
| Admin panel: 2FA | Yes. Each user sets up TOTP for their own account. No option to enforce it for all users was found. | https://coolify.io/docs/core/security/authentication/2fa | "install an authenticator application that supports time-based one-time passwords (TOTP)" |
| Admin panel: SSO | Free OAuth: GitHub, GitLab, Google, Azure, Bitbucket, Discord, Infomaniak. SSO guides: Authentik, Clerk, Zitadel. 2FA also applies to OAuth logins. | https://coolify.io/docs/core/security/authentication/oauth/overview | "For Authentik, Clerk, and Zitadel, use the SSO guides." |
| How the panel reaches Docker | Over SSH, even on its own host. The installer generates a passphrase-less ed25519 key and appends it to the installing user's `~/.ssh/authorized_keys` (root with the default installer). No Docker socket is mounted in the `coolify` container. | https://cdn.coollabs.io/coolify/install.sh | `ssh-keygen -t ed25519 -a 100 -f /data/coolify/ssh/keys/id.$CURRENT_USER@host.docker.internal -q -N ""` |
| Root SSH on localhost | The documented sshd setup allows key-only root login. | https://coolify.io/docs/knowledge-base/server/openssh | "PermitRootLogin prohibit-password allows root to connect with an SSH key" |
| Where the SSH key lives | Encrypted in Coolify's Postgres database, and also written to `/data/coolify/ssh/keys/`, which is bind-mounted into the app container | https://coolify.io/docs/core/security/credentials/ssh-keys | "Coolify encrypts the private-key value stored in its database." |
| What an attacker who compromises the panel gets | **Derived:** the app holds passphrase-less SSH keys to its own host (root by default) and to every connected server. A panel compromise therefore means root on every managed server and every app secret. Coolify's docs say the same. | https://coolify.io/docs/core/security-model | "An attacker who gains sufficient administrative access to the Coolify instance can use the same management paths." |
| Inbound ports required by default | 8000, 6001, 6002, 22, 80, 443. 8000/6001/6002 can be closed after a domain is set up. | https://coolify.io/docs/knowledge-base/server/firewall | "SSH access, unless you configured a different SSH port" |
| Cloudflare Tunnel support | Documented guides for all resources, single resource, server SSH, and full TLS. A guide that routes the Coolify dashboard itself through a tunnel was not found on the pages checked. Tailscale is available as a one-click service. | https://coolify.io/docs/integrations/cloudflare/tunnels/overview | "No need to open or forward ports on your device to the public internet." |
| Multi-server | Yes, plus dedicated build servers. The README recommends separating the Coolify server from workload servers. | https://coolify.io/docs/core/infrastructure/servers/overview | "divide workloads across multiple machines" |
| AI agents: official CLI | Yes: `coollabsio/coolify-cli` (MIT), latest v1.11.0 (2026-10-26) | https://github.com/coollabsio/coolify-cli | "CLI for Coolify" |
| AI agents: official MCP server | Yes, built into the app at `/mcp` (uses `laravel/mcp`). Off by default on self-hosted. Scoped tokens: read, read:sensitive, deploy, write, root. | https://coolify.io/docs/mcp/setup | "Until you do this, /mcp returns 404." |
| Database backups | Scheduled (cron) backups for PostgreSQL, MySQL, MariaDB, MongoDB, ClickHouse and SQLite, with optional S3-compatible upload and separate local/S3 retention. Restores are available. | https://coolify.io/docs/databases/backups | "can upload a copy to an S3-compatible storage provider." |
| One-click templates/services | "More than 360" (README). The repo's `templates/compose` holds 387 compose files (counted via GitHub API). | https://github.com/coollabsio/coolify | "more than 360 one-click services" |
| Maturity: first release | Repo created 2021-01-25. First GitHub release v1.0.0 on 2021-03-24. Latest v4.5.3 on 2026-10-28. 722 GitHub releases. | https://github.com/coollabsio/coolify/releases | not applicable (GitHub API data) |
| Maturity: GitHub stars | 65,461 | https://github.com/coollabsio/coolify | not applicable (GitHub API, 2026-11-01) |
| Maturity: contributors | 412 GitHub accounts (766 counting anonymous commit emails) | https://github.com/coollabsio/coolify/graphs/contributors | not applicable (GitHub API) |

## Where it is better than a young single-server alpha

1. **Multi-server out of the box:** separate deployment and build servers, a recommended split between the control server and workload servers, and a managed Cloud control plane from $5/month.
2. **Catalogue:** more than 360 one-click services (387 compose templates in the repo), plus Nixpacks, Railpack, Dockerfile, Compose and image deploys from GitHub, GitLab, Bitbucket or Gitea (README).
3. **Data protection:** scheduled backups for six database engines, with S3-compatible upload, retention rules and restore. A young alpha has to build all of this.
4. **Team and identity features:** teams, TOTP 2FA, free OAuth with seven providers, SSO guides (Authentik, Clerk, Zitadel), and scoped API tokens.
5. **Agent tooling and track record:** a built-in MCP server with scoped tokens, an official CLI, about 5.5 years of releases (722 GitHub releases), 65k stars and more than 400 contributors.

## Suggested verdict

Brume vs Coolify: pick Coolify for breadth and maturity (multi-server, 360+ services, S3 backups, SSO, MCP). Pick Brume for a smaller blast radius on one server: no public panel and no root SSH key held by the web app.
