<p align="center">
  <img src="assets/logo.png" alt="Brume" width="160">
</p>

<h1 align="center">Brume</h1>

<p align="center">
  A source-available, self-hostable alternative to Railway, Coolify and Dokploy, with a simpler UX and developer experience.
</p>

> [!WARNING]
> **Brume is an alpha.** It installs on a VPS with one command, and deploys public Docker images with variables, secrets, logs and a public domain. It has no backups and no S3 yet, so a lost server cannot be restored. It does not update itself, signs you in with a one-time link instead of a passkey, and does not build from GitHub. Do not run anything you cannot lose on it, and expect everything to change.

## Goals for alpha 0.1

- A single VPS, a single owner.
- Debian 12 or 13, Ubuntu 24.04 or 26.04, on amd64 or arm64.
- One-command install: `curl -sSL https://brume.dev/install.sh | sudo bash -s -- --domain <zone>`.
- Deploy any container, plus static sites with instant rollback.
- Templates with good practices built in: static site, PostgreSQL, MongoDB, Redis.
- Automatic CI/CD from GitHub, built with Railpack.
- Backups, logs and secrets managed from the panel.
- Admin panel over Tailscale by default. Access through an SSH tunnel or from the public Internet is opt-in.
- Agent first: a CLI and an MCP server, reached the same way as the panel. Deploy anything in one command, or in one sentence to your AI agent.
- Public traffic through Cloudflare Tunnel by default: no inbound port open on the server. Direct ingress on ports 80 and 443 is opt-in.
- All of Brume's state on S3, with its secrets encrypted: a lost VPS is rebuilt from the bucket.

**Planned stack:** Rust (tokio, clap), React (TanStack Query, Router and DB, coss ui, Tailwind, React Flow), SQLite, DuckDB, S3, OpenTelemetry, Docker Engine, Traefik, Caddy, Cloudflare Tunnel, Tailscale.

## Design documents

The architecture decisions live in [`archi/base/`](archi/base/README.md) (written in French): threat model, architecture, data model, networking, admin panel, runtime, templates, builds and deployment, secrets and backups, host setup, observability, development rules, a comparison with similar tools, the software architecture (engine and agent, event sourcing), state and restore on S3, deterministic simulation testing, telemetry, and the CLI and MCP server for AI agents.

## Documentation

The public docs live in [`docs/`](docs/), a static [Fumadocs](https://fumadocs.dev) site in three parts: using Brume, how it works, and customizing it. Run them with `pnpm install && pnpm dev` in `docs/`. Writing rules for people and agents: [`docs/AGENTS.md`](docs/AGENTS.md).

## Code style

Rust code follows the [Google Rust style guide](https://google.github.io/styleguide/rustguide.html), with a few Brume-specific adaptations listed in [`archi/base/10-developpement.md`](archi/base/10-developpement.md#style-rust). Format with `cargo +nightly fmt`.

## Status

**Milestone 2 of alpha 0.1: install it on a VPS.** On a fresh server running Ubuntu 24.04 or 26.04, or Debian 12 or 13:

```sh
curl -sSL https://brume.dev/install.sh | sudo bash -s -- --domain <zone>
```

`install.sh` downloads the archive for the server's architecture from a GitHub pre-release and checks it against the SHA-256 it pins. `brume install` then sets up a firewall (nftables), Docker with user namespaces, one system user per service, Tailscale with the panel on your tailnet, and with `--domain`, a Cloudflare tunnel for your apps' public domains. It ends with a one-time login link to the panel. See the [quickstart](docs/content/docs/use/quickstart.mdx), and [`archi/jalons/2026-11-01-jalon-2-vps.md`](archi/jalons/2026-11-01-jalon-2-vps.md) (in French) for the scope of the milestone. It was tested on Ubuntu 24.04 on arm64: Debian and amd64 are built, but not yet tested on a VM.

**Milestone 1: run it locally.** `brume dev` starts the engine, the agent, telemetry and the API on your machine, against your local Docker. See [Run Brume locally](docs/content/docs/customize/run-locally.mdx).

**Not there yet:** the S3 bucket, backups and restore, the age keys, minisign signatures and the updater, the passkey, `tag:brume`, closing port 22, the read-only Cloudflare token and HSTS, GitHub builds, templates, static sites, and the CLI on your laptop. Open questions are tracked in [`archi/base/README.md`](archi/base/README.md#décisions-ouvertes).

## License

Brume is source-available under the [Functional Source License, Version 1.1, ALv2 Future License](LICENSE.md) (FSL-1.1-ALv2). You can use it, change it and self-host it, including in a business, for any purpose except a competing product or service. Each version becomes available under the Apache License 2.0 two years after its release.
