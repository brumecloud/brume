# Brume vs Dokku

Checked: 2026-11-01. Official sources only (dokku.com/docs, the dokku/dokku GitHub repo and official plugin repos, pro.dokku.com). The Docker docs are cited for the docker-group fact.

**What it is:** Dokku is a free, MIT-licensed, Heroku-style PaaS (since 2013). You install it on one server, deploy with `git push` over SSH, and drive it from the command line. A web UI and API exist only in the paid Dokku Pro.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| License | MIT (core). Dokku Pro is a separate commercial product. | https://github.com/dokku/dokku/blob/master/LICENSE | "Permission is hereby granted, free of charge, to any person obtaining a copy of this software" |
| Pricing | Core is free. Dokku Pro (web UI + JSON API) costs $849 one time, lifetime, with free upgrades. | https://pro.dokku.com/ | "It's a one-time lifetime license with free upgrades forever." |
| Dokku Pro license scope | One production server plus two pre-production servers. Volume pricing on request. | https://pro.dokku.com/ | "One production server plus two pre-production servers." |
| Dokku Pro license check | The server needs public internet access to validate the license | https://dokku.com/docs/enterprise/pro/ | "the server must be able to contact the public internet to validate the license" |
| Install method | `bootstrap.sh` from dokku.com with sudo, which installs via apt. Docker-image and other methods are also documented. | https://dokku.com/docs/getting-started/installation/ | "wget -NP . https://dokku.com/install/v0.38.33/bootstrap.sh" (the site shows v0.38.33; GitHub latest is v0.38.34) |
| Server requirements | Ubuntu 22.04/24.04/26.04 or Debian 11+, amd64 or arm64. 1 GB RAM with the Docker scheduler, 2 GB per node with K3s. | https://dokku.com/docs/getting-started/installation/ | "Docker Scheduler: 1GB of system memory, or add swap memory" |
| Scope (core) | Single server by design | https://dokku.com/docs/getting-started/installation/ | "Dokku is an extensible, open source Platform as a Service that runs on a single server of your choice." |
| Admin interface (core) | CLI. Commands run on the host or over SSH (`ssh -t dokku@host <command>`), and there is an official bash client. No web UI in core. | https://dokku.com/docs/deployment/remote-commands/ | "Dokku commands can be run over SSH." |
| Web UI (Dokku Pro only) | Paid SPA plus JSON API, both with JWT auth. Default port 5000. | https://pro.dokku.com/docs/getting-started/installation/ | "By default the service listens on port `5000`." |
| Web UI public by default? | Not found. The docs show putting Pro behind Dokku's nginx with a domain and Let's Encrypt. Pro serves plain HTTP only. | https://pro.dokku.com/docs/operating/exposing-publicly/ | "serves plain `http` and cannot terminate `https` itself" |
| Web UI auth options | Username and password (built-in `root` user plus users and teams), and reverse-proxy auth (oauth2-proxy, Authelia, Authentik, Pomerium) for SSO-style login. 2FA: not found. | https://pro.dokku.com/docs/features/reverse-proxy-authentication/ | "Any client that can reach Dokku Pro directly can forge the header and sign in as anyone." |
| SSH user model (core) | Keys go in the `dokku` user's `authorized_keys`. Per-user command scoping needs a plugin. | https://dokku.com/docs/deployment/user-management/ | "Support for scoping commands to specific users can be added through plugins" |
| How it reaches Docker | Dokku runs on the host. The package adds the `dokku` user to the `docker` group, and the default scheduler talks to the local Docker socket. The Docker-based install mounts `/var/run/docker.sock`. | https://github.com/dokku/dokku/blob/master/debian/postinst | "usermod -aG docker dokku" |
| Default scheduler | docker-local | https://dokku.com/docs/deployment/schedulers/scheduler-management/ | "Schedules apps against the local docker socket and runs containers directly on the Dokku host." |
| Docker group = root | Docker's own docs (used for the derivation below) | https://docs.docker.com/engine/install/linux-postinstall/ | "The `docker` group grants root-level privileges to the user." |
| What an attacker gets by compromising the control interface | **Derived.** Any key in `dokku`'s `authorized_keys`, or a Dokku Pro admin login, can run every Dokku command on every app; scoping needs a plugin. Those commands can pass arbitrary `docker run` options (`docker-options:add`), and the `dokku` user is in the `docker` group, which Docker calls root-level. So a stolen admin key or Pro session is effectively root on the host. | https://dokku.com/docs/advanced-usage/docker-options/ | "docker-options:add [--process PROC...] <app> <phase(s)> OPTION # Add Docker option to app for phase" |
| Inbound ports by default | SSH 22 (git push and remote commands go over SSH as `dokku@host`) and HTTP/HTTPS 80/443 (the nginx proxy). If installed, Dokku Pro listens on 5000; whether that port is public by default is not stated. | https://dokku.com/docs/deployment/application-deployment/ | "git remote add dokku dokku@dokku.me:ruby-getting-started" |
| SSH-less deploy option | Only in Dokku Pro, through HTTPS git push | https://dokku.com/docs/enterprise/pro/ | "an alternative, https-based method for deploying code which can be used in environments that lockdown ssh access" |
| Cloudflare Tunnel support | Not found in the official docs or the official plugin list. A community Tailscale plugin is listed. | https://dokku.com/docs/community/plugins/ | not found |
| Multi-server support | Yes. The built-in K3s scheduler (since 0.33.0) builds a multi-node cluster. Official Nomad and Kubernetes scheduler plugins also exist. Dokku Pro has a "Managing Multiple Servers" feature. | https://dokku.com/docs/deployment/schedulers/k3s/ | "For multi-server support, Dokku provides the ability for users to setup a K3s cluster." |
| AI agents: official CLI | Yes. `dokku` itself, plus the official remote client `contrib/dokku_client.sh` (over SSH). | https://dokku.com/docs/deployment/remote-commands/ | "Of all methods, this is the *most* official method of interacting with your Dokku installation." |
| AI agents: official MCP server | Not found (none in the docs or the dokku GitHub org) | https://github.com/dokku | not found |
| Database backups (to S3?) | Yes. The official datastore plugins back up to an S3 bucket on a schedule, with optional GPG encryption. `backup-auth` takes an endpoint URL, so S3-compatible stores work. | https://github.com/dokku/dokku-postgres | "create a backup of the Postgres service to an existing s3 bucket" |
| Templates / plugin ecosystem | No one-click app template catalog (not found). 28 official plugins (Postgres, MySQL, MariaDB, Redis, Mongo, Let's Encrypt, ...) plus a list of 55 active community plugins. | https://dokku.com/docs/community/plugins/ | "The following plugins are available and provided by Dokku maintainers." |
| Swappable proxy | nginx by default. Caddy, Traefik, HAProxy and OpenResty are supported. | https://dokku.com/docs/networking/proxy-management/ | "The default proxy shipped with Dokku is `nginx`. It can be changed via the `proxy:set` command." |
| Maturity: first release | 2013 (GitHub repo created 2013-06-08, first tag v0.1.0). Still 0.x: latest v0.38.34, released 2026-10-30. | https://github.com/dokku/dokku/releases | not applicable |
| Maturity: GitHub stars | 32,216 (GitHub API, 2026-11-01) | https://github.com/dokku/dokku | not applicable |

## Where it is better than a young single-server alpha

1. **About 13 years in production.** Public since 2013, 32k GitHub stars, and still shipping often (v0.38.34 on 2026-10-30).
2. **Mature datastore story.** 28 official plugins cover Postgres, MySQL, MariaDB, Redis, Mongo, Elasticsearch, RabbitMQ and more, with scheduled, optionally GPG-encrypted backups to S3 or S3-compatible storage.
3. **Heroku-style workflow.** `git push` deploys with buildpacks or a Dockerfile, plus Procfile process types, cron tasks, process scaling and a swappable proxy (nginx, Caddy, Traefik, HAProxy, OpenResty).
4. **Room to grow past one box.** A built-in K3s scheduler for multi-node clusters, plus official Nomad and Kubernetes scheduler plugins. Brume 0.1 is single-server.
5. **Free MIT core** with an optional one-time-fee UI ($849 lifetime). Dokku Pro adds a JSON API, teams and reverse-proxy SSO.

## Suggested verdict

**Brume vs Dokku:** pick Dokku for a free, battle-tested git-push PaaS driven over SSH, with a deep plugin and datastore ecosystem and a path to multi-node. Pick Brume for a web panel out of the box, no public SSH or HTTP ports, and a control plane that never holds the Docker socket.
