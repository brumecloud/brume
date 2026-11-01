<p align="center">
  <img src="assets/logo.png" alt="Brume" width="160">
</p>

<h1 align="center">Brume</h1>

<p align="center">
  A source-available, self-hostable alternative to Railway, Coolify and Dokploy, with a simpler UX and developer experience.
</p>

> [!WARNING]
> **Brume is a work in progress.** The first milestone of alpha 0.1 runs locally: from the panel, on your own machine, you deploy public Docker images with variables and secrets, and watch their logs. Nothing is exposed to the Internet, there is no installer yet, and everything is subject to change.

## Goals for alpha 0.1

- A single VPS, a single owner.
- Debian 12 or 13, Ubuntu 24.04 or 26.04, on amd64 or arm64.
- One-command install: `curl -sSL https://github.com/brumecloud/brume/releases/latest/download/install.sh | bash`.
- Deploy any container, plus static sites with instant rollback.
- Templates with good practices built in: static site, PostgreSQL, MongoDB, Redis.
- Automatic CI/CD from GitHub, built with Railpack.
- Backups, logs and secrets managed from the panel.
- Admin panel over Tailscale by default. Access through an SSH tunnel or from the public Internet is opt-in.
- Public traffic through Cloudflare Tunnel by default: no inbound port open on the server. Direct ingress on ports 80 and 443 is opt-in.
- All of Brume's state on S3, encrypted: a lost VPS is rebuilt from the bucket.

**Planned stack:** Rust (tokio, clap), React (TanStack Query, Router and DB, coss ui, Tailwind, React Flow), SQLite, DuckDB, S3, OpenTelemetry, Docker Engine, Traefik, Caddy, Cloudflare Tunnel, Tailscale.

## Design documents

The architecture decisions live in [`archi/base/`](archi/base/README.md) (written in French): threat model, architecture, data model, networking, admin panel, runtime, templates, builds and deployment, secrets and backups, host setup, observability, development rules, a comparison with similar tools, the software architecture (engine and agent, event sourcing), state and restore on S3, deterministic simulation testing, and telemetry.

## Documentation

The public docs live in [`docs/`](docs/), a static [Fumadocs](https://fumadocs.dev) site in three parts: using Brume, how it works, and customizing it. Run them with `pnpm install && pnpm dev` in `docs/`. Writing rules for people and agents: [`docs/AGENTS.md`](docs/AGENTS.md).

## Code style

Rust code follows the [Google Rust style guide](https://google.github.io/styleguide/rustguide.html), with a few Brume-specific adaptations listed in [`archi/base/10-developpement.md`](archi/base/10-developpement.md#style-rust). Format with `cargo +nightly fmt`.

## Status

**Milestone 1 of alpha 0.1: run it locally.** `brume dev` starts the engine, the agent, telemetry and the API on your machine, against your local Docker, and prints a login link to the panel on http://127.0.0.1:7878. See [Run Brume locally](docs/content/docs/customize/run-locally.mdx) for the steps, and [`archi/jalons/2026-10-31-jalon-1-deploiement-local.md`](archi/jalons/2026-10-31-jalon-1-deploiement-local.md) (in French) for the scope of the milestone and what it leaves out.

The rest of alpha 0.1 (installer, Tailscale, Cloudflare Tunnel, GitHub builds, templates, backups, S3) is still design. Open questions are tracked in [`archi/base/README.md`](archi/base/README.md#décisions-ouvertes).

## License

Brume is source-available under the [Functional Source License, Version 1.1, ALv2 Future License](LICENSE.md) (FSL-1.1-ALv2). You can use it, change it and self-host it, including in a business, for any purpose except a competing product or service. Each version becomes available under the Apache License 2.0 two years after its release.
