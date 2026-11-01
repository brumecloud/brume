// What each component of the diagram explains. Every fact comes from a page of
// docs/content/docs/how-it-works/, named in `docs`.

import type { ArchComponent, ComponentId } from "./model";

const componentData = [
  {
    id: "internet",
    label: "Internet",
    detail: "visitors and GitHub",
    kind: "external",
    icon: { kind: "glyph", name: "globe" },
    column: 0,
    row: 0,
    role: "Your visitors and GitHub. They reach your apps through Cloudflare.",
    why: "Your users and GitHub's webhooks come from here. Nothing on your server waits for them: they reach your apps through Cloudflare.",
    how: [
      "Visitors open your apps on your domain, over HTTPS.",
      "GitHub calls Brume's webhook when you push to a configured branch.",
      "DNS points to Cloudflare, never to your server's IP.",
    ],
    rules: [
      {
        rule: "No inbound port.",
        why: "The firewall denies all inbound traffic, SSH included. Scanning your server's IP finds nothing to attack.",
      },
      {
        rule: "A leaked IP doesn't matter.",
        why: "Nothing listens on it.",
      },
    ],
    docs: "how-it-works/networking",
  },
  {
    id: "cloudflare",
    label: "Cloudflare Tunnel",
    detail: "no inbound port",
    kind: "host",
    icon: { kind: "brand", name: "cloudflare" },
    column: 1,
    row: 0,
    role: "Brings public traffic in, without opening a single port.",
    why: "It brings public traffic in without opening a port. cloudflared, on your server, dials out to Cloudflare, and requests come back through that connection.",
    how: [
      "Cloudflare holds the public certificates, HSTS and the redirect to HTTPS: no Let's Encrypt, no port 80.",
      "cloudflared sends everything to Traefik, which answers 404 for unknown domains.",
      "A wildcard on your domain points to the tunnel, so a new service needs no DNS change.",
      "The trade-off: Cloudflare decrypts your public traffic. Direct ingress on ports 80 and 443 is the alternative, chosen on the host.",
    ],
    rules: [
      {
        rule: "Tunnel config managed on the server.",
        why: "A route added in the Cloudflare dashboard must not divert your traffic. Whether a local config really blocks dashboard routes is still being checked.",
      },
      {
        rule: "Unknown connectors raise an alert.",
        why: "A stolen tunnel secret lets someone run a second connector. A read-only token lets the engine spot it and alert you.",
      },
      {
        rule: "API token forgotten after install.",
        why: "Only the tunnel secret and a read-only token stay on the server, and the token can't change anything.",
      },
    ],
    ifCompromised:
      "Reading and modifying public traffic. cloudflared reaches only Traefik and the webhook.",
    risk: "Your public traffic. It reaches only Traefik and the webhook.",
    docs: "how-it-works/networking",
  },
  {
    id: "traefik",
    label: "Traefik",
    detail: "routes to your apps",
    kind: "host",
    icon: { kind: "brand", name: "traefik" },
    column: 2,
    row: 0,
    role: "Routes each public request to the right app.",
    why: "It routes every public request to the right app. It handles all public traffic, so it runs with as few rights as possible.",
    how: [
      "Listens only on its fixed IP in the ingress network.",
      "Reaches each exposed app through a network shared with that app alone.",
      "The agent writes its config to a folder Traefik mounts read-only.",
    ],
    rules: [
      {
        rule: "Never the Docker socket.",
        why: "A router holding the socket is root on the host. Its config comes from files instead.",
      },
      {
        rule: "On no environment network.",
        why: "So it never reaches your databases.",
      },
      {
        rule: "Forwarded headers from cloudflared only.",
        why: "Otherwise a client could show your apps a spoofed IP.",
      },
      {
        rule: "Webhook path: GitHub IPs only.",
        why: "It cuts the noise. The webhook's signature check stays the real barrier.",
      },
      {
        rule: "No port, no root, read-only.",
        why: "Every public request goes through it.",
      },
    ],
    ifCompromised:
      "Reading and modifying public traffic, and reaching the exposed services. Not the databases.",
    risk: "Public traffic and the exposed apps. Never the databases.",
    docs: "how-it-works/networking",
  },
  {
    id: "apps",
    label: "Your apps",
    detail: "containers",
    kind: "host",
    icon: { kind: "glyph", name: "apps" },
    column: 3,
    row: 0,
    role: "Your code, with one isolated network per environment.",
    why: "Your code, in containers. Each environment is its own network, cut off from the others.",
    how: [
      "Deployed by image digest, so a moved tag never changes production.",
      "Without a volume, a new version starts next to the old one, and traffic switches once its healthcheck passes. With a volume, the service stops, then restarts.",
      "Secrets arrive as environment variables when the container starts.",
    ],
    rules: [
      {
        rule: "No outbound internet by default.",
        why: "You turn egress on per service, so a compromised app can't call out unless you allowed it.",
      },
      {
        rule: "No route to your tailnet or private networks.",
        why: "nftables rules stop a compromised container from using your server to reach your other machines.",
      },
      {
        rule: "Root in a container is not root on the host.",
        why: "userns-remap maps it to an unprivileged user, so an escape doesn't give root directly.",
      },
      {
        rule: "Dangerous options off, unless root allows them.",
        why: "--privileged, host mounts or the Docker socket need an edit to policy.toml, never a click in the panel.",
      },
    ],
    ifCompromised: "Its environment, nothing else.",
    risk: "Its own environment, nothing else.",
    docs: "how-it-works/runtime",
  },
  {
    id: "caddy",
    label: "Caddy",
    detail: "static sites",
    kind: "host",
    icon: { kind: "brand", name: "caddy" },
    column: 3,
    row: 1,
    role: "Serves your static sites, with instant rollbacks.",
    why: "It serves your static sites. Recent versions stay on disk, so a rollback is instant.",
    how: [
      "Each domain serves a link to the folder of its revision. A deploy or a rollback switches the link at once.",
      "TLS ends at Cloudflare, so Caddy's automatic HTTPS is off.",
      "Its config is JSON generated by Brume, mounted read-only.",
    ],
    rules: [
      {
        rule: "Read-only files, no root.",
        why: "A compromised Caddy can't rewrite your sites on disk.",
      },
      {
        rule: "Hidden files are refused.",
        why: "Builds often leave .env or .git in the output folder.",
      },
      {
        rule: "Archives unpacked without privileges.",
        why: "Paths with ../ and symlinks are rejected, so Caddy can't serve files outside the site.",
      },
      {
        rule: "Admin API locked in its container.",
        why: "The policy forbids any other container from sharing its network.",
      },
    ],
    ifCompromised: "Serving modified content until it restarts.",
    risk: "Modified pages, until it restarts.",
    docs: "how-it-works/runtime",
  },
  {
    id: "webhook",
    label: "brume webhook",
    detail: "GitHub pushes",
    kind: "brume",
    icon: { kind: "brume" },
    column: 2,
    row: 1,
    role: "Takes GitHub's pushes. The only Brume part on the internet.",
    why: "It receives GitHub's push events. It's the only Brume component reachable from the internet, so it's a separate, tiny process.",
    how: [
      "Checks the HMAC signature before it parses anything.",
      "Turns a push into a single call to the engine: enqueue_build(repo_id, ref).",
      "The repository comes from Brume's config, never from the payload.",
    ],
    rules: [
      {
        rule: "Separate from the API.",
        why: "Inside the API, a flaw in its parsing would give every admin action.",
      },
      {
        rule: "Signature first, in constant time.",
        why: "It parses no unsigned data from the internet.",
      },
      {
        rule: "Duplicate deliveries are ignored.",
        why: "GitHub signs no timestamp, so this stops replays.",
      },
      {
        rule: "One operation, no outbound network.",
        why: "Even a hijacked webhook can only queue a build of a repository you configured.",
      },
    ],
    ifCompromised: "Queuing a build of an already configured repository.",
    risk: "Queuing a build of a repo you configured. That's all.",
    docs: "how-it-works/builds-and-deployments",
  },
  {
    id: "tailnet",
    label: "Your tailnet",
    detail: "you, the CLI, AI agents",
    kind: "external",
    icon: { kind: "brand", name: "tailscale" },
    column: 0,
    row: 2,
    role: "Your private way in: the panel, the CLI and SSH.",
    why: "The way in for administration. The panel, the CLI and SSH live on your private Tailscale network, not on the internet.",
    how: [
      "tailscale serve exposes the panel over HTTPS, on a *.ts.net name.",
      "It adds your Tailscale identity to each request. Brume checks it against an allowlist set at install.",
      "Your databases can be opened to your laptop over TCP, one by one, never through a public port.",
    ],
    rules: [
      {
        rule: "Being on the tailnet is not enough.",
        why: "A compromised device or a shared tailnet would get in, so the panel also asks for a passkey.",
      },
      {
        rule: "Your policy must keep it off your devices.",
        why: "A tailnet's default policy allows everything, so yours must forbid it: then a compromised server can't reach your laptop.",
      },
      {
        rule: "Never Tailscale Funnel.",
        why: "Funnel would publish the panel on the internet.",
      },
    ],
    ifCompromised:
      "Your Tailscale identity alone gives a root shell through Tailscale SSH, an accepted risk in 0.1. The panel still asks for your passkey.",
    risk: "A root shell through Tailscale SSH, an accepted 0.1 risk. The panel still asks for your passkey.",
    docs: "how-it-works/admin-panel",
  },
  {
    id: "api",
    label: "brume api",
    detail: "the panel",
    kind: "brume",
    icon: { kind: "brume" },
    column: 1,
    row: 2,
    role: "Serves the panel. The engine decides, not the API.",
    why: "It serves the panel and its API, to your browser and to the CLI. Every request goes on to the engine, which decides.",
    how: [
      "Listens only on 127.0.0.1. tailscale serve brings it to your tailnet.",
      "Forwards each request, with its session or its CLI signature, to the engine over a unix socket.",
      "Streams logs to the panel, once the engine has checked your session.",
    ],
    rules: [
      {
        rule: "Two factors: Tailscale identity and passkey.",
        why: "Being on your tailnet alone isn't enough.",
      },
      {
        rule: "The engine checks every session.",
        why: "A compromised API can do nothing without a session you opened.",
      },
      {
        rule: "CLI requests are signed on your device.",
        why: "The API can neither forge nor change them, only drop them.",
      },
      {
        rule: "Strict CSP, same-origin only.",
        why: "The panel shows hostile content, such as log lines and commit messages. It must never run in your browser.",
      },
      {
        rule: "Localhost only, no outbound network.",
        why: "Code running in it can neither download tools nor send data out.",
      },
    ],
    ifCompromised:
      "Nothing without a session opened by you. During a session, the panel's actions, without the dangerous options.",
    risk: "Nothing without a session you opened.",
    docs: "how-it-works/admin-panel",
  },
  {
    id: "s3",
    label: "S3 bucket",
    detail: "state and backups",
    kind: "external",
    icon: { kind: "glyph", name: "storage" },
    column: 0,
    row: 3,
    role: "Holds your state and backups, so the server is disposable.",
    why: "Your server is disposable. Brume's state and your database dumps live in your bucket, so a new server comes back from it.",
    how: [
      "The engine streams its event log there, with the secrets encrypted.",
      "The agent sends database dumps, encrypted with your age key, after test-restoring each one.",
      "Any S3-compatible storage. Another provider than your VPS is advised.",
    ],
    rules: [
      {
        rule: "Brume can't delete anything.",
        why: "Its credentials can't delete, change lifecycle rules or suspend versioning. A compromised server can't erase what already left.",
      },
      {
        rule: "Encrypted for a key the server lacks.",
        why: "The server can encrypt them, never read them.",
      },
      {
        rule: "Versioning or Object Lock required.",
        why: "An overwritten object keeps its original.",
      },
      {
        rule: "Bucket changed on the host only.",
        why: "A stolen session can't send your next backups somewhere else.",
      },
    ],
    ifCompromised:
      "Whoever has the S3 credential reads the event log, which shows how you are set up. Secrets and backups stay encrypted. They could also write fake events that a restore would replay, an accepted risk in 0.1.",
    risk: "The shape of your setup. Secrets and backups stay encrypted.",
    docs: "how-it-works/secrets-and-backups",
  },
  {
    id: "engine",
    label: "brume engine",
    detail: "state and secrets",
    kind: "brume",
    icon: { kind: "brume" },
    column: 2,
    row: 3,
    role: "Decides every command and holds the secrets. Never touches Docker.",
    why: "The control plane. It decides every command and owns the state and the secrets, but never touches Docker.",
    how: [
      "Keeps the state as an event log in SQLite, and ships it to S3 in the background.",
      "Checks every request: the session or the CLI signature, then the policy.",
      "Sends the agent the full desired state of its machine, never a diff.",
    ],
    rules: [
      {
        rule: "No Docker socket.",
        why: "A compromised engine doesn't give root on the host without a kernel or runc flaw.",
      },
      {
        rule: "No untrusted data.",
        why: "No git clone, no archive, no webhook or log parsing: each parser would be a door to the secrets.",
      },
      {
        rule: "Unix sockets only, one per client.",
        why: "File permissions decide who talks to it, and each socket has its own operations.",
      },
      {
        rule: "Master key in systemd credentials.",
        why: "Only the engine gets it at startup. Other services can't read it, even through a bug that reads files.",
      },
    ],
    ifCompromised:
      "The apps, their secrets and their data, within the limits of policy.toml. Not root on the host without a kernel or runc flaw.",
    risk: "Your apps and their secrets. Not root on the host.",
    docs: "how-it-works/architecture",
  },
  {
    id: "updater",
    label: "brume updater",
    detail: "signed updates",
    kind: "brume",
    icon: { kind: "brume" },
    column: 3,
    row: 2,
    role: "Installs signed updates, and nothing else.",
    why: "It installs new versions of Brume. It runs only during an update, and accepts only signed binaries.",
    how: [
      "The engine reads brume.dev/latest.json, a manifest signed offline, and starts the updater when a newer version exists.",
      "The updater checks everything again: the signature, the version and the hash of the binary.",
      "Then it replaces the binary and restarts the services.",
    ],
    rules: [
      {
        rule: "It doesn't trust the engine.",
        why: "A compromised engine can't make it install an unsigned binary.",
      },
      {
        rule: "Never backwards.",
        why: "An old, vulnerable version stays signed, so the updater refuses any version that is not newer.",
      },
      {
        rule: "Two public keys built in.",
        why: "If the signing key is lost or stolen, a release signed with the backup key, kept offline, replaces it.",
      },
      {
        rule: "The binaries belong to root.",
        why: "A compromised service, other than the agent, can't persist by replacing its own binary.",
      },
    ],
    ifCompromised:
      "Equivalent to root, but it runs only during an update and accepts only signed binaries.",
    risk: "Root, but only during an update, and only signed binaries.",
    docs: "how-it-works/host-and-installation",
  },
  {
    id: "telemetry",
    label: "brume telemetry",
    detail: "logs, in DuckDB",
    kind: "brume",
    icon: { kind: "brume" },
    column: 3,
    row: 3,
    role: "Stores and searches your logs, away from the rest.",
    why: "It stores and searches the logs, apart from the rest, so a flood of logs can't slow the engine down.",
    how: [
      "Docker sends each container's logs to it, over a unix socket only root can write to.",
      "The panel reads logs through the engine, which checks your session first.",
      "Logs are kept for a limited time: apps sometimes write secrets in them.",
    ],
    rules: [
      {
        rule: "No network at all.",
        why: "Even a successful injection gets no network, no secrets and no Docker.",
      },
      {
        rule: "A hardened DuckDB.",
        why: "Parameterized queries, no file access, no extensions, and settings locked once set.",
      },
      {
        rule: "Memory and CPU caps.",
        why: "A flood of logs can't starve the server.",
      },
    ],
    ifCompromised: "Reading the logs and the telemetry. No network, no secrets, no Docker.",
    risk: "Your logs. No network, no secrets, no Docker.",
    docs: "how-it-works/observability",
  },
  {
    id: "agent",
    label: "brume agent",
    detail: "the Docker socket",
    kind: "brume",
    icon: { kind: "brume" },
    column: 2,
    row: 4,
    role: "The only part that drives Docker. It listens on nothing.",
    why: "The only part of Brume that drives Docker. It makes the engine's desired state real, and enforces its own policy.",
    how: [
      "A reconciliation loop: after a crash or a reboot, it puts everything back on its own.",
      "Builds each container's config from an allowlist, and copies nothing from a request.",
      "Runs the builds and the backups, and reports drift, such as a container started by hand.",
    ],
    rules: [
      {
        rule: "It listens on nothing.",
        why: "It connects to the engine. Nothing can connect to it.",
      },
      {
        rule: "It doesn't trust the engine.",
        why: "It enforces policy.toml on every container, even against a compromised engine.",
      },
      {
        rule: "Minimal code, no untrusted data.",
        why: "With the Docker socket, it is as powerful as root. The git clone and archive extraction run in unprivileged processes.",
      },
    ],
    ifCompromised:
      "Everything, on its machine. With the updater, it is the only Brume component in this position, hence its minimal code.",
    risk: "Everything on its machine, hence its minimal code.",
    docs: "how-it-works/architecture",
  },
  {
    id: "docker",
    label: "Docker",
    detail: "your containers",
    kind: "host",
    icon: { kind: "brand", name: "docker" },
    column: 3,
    row: 4,
    role: "Runs the containers. Only the agent holds its socket.",
    why: "The container runtime. Whoever holds its socket is root, so in Brume only the agent does.",
    how: [
      "Runs your apps, Traefik, Caddy, cloudflared and the builder.",
      "Sends each container's logs to brume telemetry, never through the engine.",
      "Version 28.5.2 or later, with the runc fixes of November 2025.",
    ],
    rules: [
      {
        rule: "No container publishes a port.",
        why: "Docker bypasses the firewall: a published port would be exposed despite it.",
      },
      {
        rule: "userns-remap is on.",
        why: "Root in a container maps to an unprivileged user on the host.",
      },
      {
        rule: "Default seccomp and AppArmor, never off.",
        why: "The one exception is the builder, an accepted risk.",
      },
      {
        rule: "nftables rules load before Docker.",
        why: "Whatever rules Docker adds, a container can't reach your tailnet, private networks or the metadata endpoint.",
      },
    ],
    docs: "how-it-works/runtime",
  },
  {
    id: "buildkit",
    label: "BuildKit",
    detail: "builds your images",
    kind: "host",
    icon: { kind: "brand", name: "docker" },
    column: 2,
    row: 5,
    role: "Builds your images, away from the engine and the agent.",
    why: "It turns your repository into an image with Railpack, in its own container, so neither the engine nor the agent runs your build.",
    how: [
      "Clones with a GitHub token limited to that repository, valid for one hour.",
      "Build secrets go through --secret, never ARG or ENV, and are masked in the logs.",
      "Produces an image. The agent loads it and deploys it by digest.",
    ],
    rules: [
      {
        rule: "Rootless, no Docker socket.",
        why: "A build can't drive the host's Docker.",
      },
      {
        rule: "One cache per environment.",
        why: "A compromised staging build can't poison production's cache.",
      },
      {
        rule: "One build at a time, with limits.",
        why: "A heavy build can't starve your apps or Brume.",
      },
      {
        rule: "Cut off from your tailnet.",
        why: "Builds need npm or pip, not your machines.",
      },
    ],
    ifCompromised:
      "The current build, its secrets, the GitHub token and its environment's cache. Its sandbox is weaker than an app's, so with a kernel or runc flaw, a path to root: an accepted risk, since it builds your own code.",
    risk: "The current build and its secrets. With a kernel flaw, root.",
    docs: "how-it-works/builds-and-deployments",
  },
] as const satisfies readonly ArchComponent[];

/** Fails the type check when a `ComponentId` has no entry above. */
type AssertNever<T extends never> = T;
export type MissingComponents = AssertNever<
  Exclude<ComponentId, (typeof componentData)[number]["id"]>
>;

export const archComponents: readonly ArchComponent[] = componentData;

const byId = new Map<ComponentId, ArchComponent>(
  archComponents.map((component) => [component.id, component]),
);

export function componentById(id: ComponentId): ArchComponent {
  const found = byId.get(id);
  if (!found) throw new Error(`No component ${id} in the architecture data`);
  return found;
}
