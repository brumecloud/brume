<p align="center">
  <img src="assets/logo.png" alt="Brume" width="160">
</p>

<h1 align="center">Brume</h1>

<p align="center">
  An open source, self-hostable alternative to Railway, Coolify and Dokploy, with a simpler UX and developer experience.
</p>

> [!WARNING]
> **Brume is a work in progress.** There is no code yet: this repository only holds the architecture and security design for the first alpha (0.1). Nothing here is usable, and everything is subject to change.

## Goals for alpha 0.1

- A single VPS, a single owner.
- One-command install: `curl -sSL https://github.com/brumecloud/brume/releases/latest/download/install.sh | bash`.
- Deploy any container, plus static sites with instant rollback.
- Templates with good practices built in: static site, PostgreSQL, MongoDB, Redis.
- Automatic CI/CD from GitHub, built with Railpack.
- Backups, logs and secrets managed from the panel.
- Admin panel reachable only over Tailscale.
- Public traffic only through Cloudflare Tunnel: no inbound port open on the server.

**Planned stack:** Rust (tokio, clap), React (TanStack Query, Router and DB, shadcn, Tailwind), SQLite, DuckDB, Docker Engine, Traefik, nginx, Cloudflare Tunnel, Tailscale.

## Design documents

The architecture decisions live in [`archi/base/`](archi/base/README.md) (written in French): threat model, architecture, data model, networking, admin panel, runtime, templates, builds and deployment, secrets and backups, host setup, observability, development rules, and a comparison with similar tools.

## Code style

Rust code follows the [Google Rust style guide](https://google.github.io/styleguide/rustguide.html), with a few Brume-specific adaptations listed in [`archi/base/10-developpement.md`](archi/base/10-developpement.md#style-rust). Format with `cargo +nightly fmt`.

## Status

Brume is in the design phase. Open questions are tracked in [`archi/base/README.md`](archi/base/README.md#décisions-ouvertes).
