// Links and copy shared across the landing page.

const repo = "https://github.com/brumecloud/brume";

export const site = {
  name: "Brume",
  url: "https://brume.dev",
  title: "Brume · Your apps online, your server invisible",
  description:
    "Brume is an open source, self-hosted platform to deploy containers, static sites and databases on your own server, with zero inbound ports.",
  repo,
  designDocs: `${repo}/tree/main/archi/base`,
  issues: `${repo}/issues`,
  installCommand: `curl -sSL ${repo}/releases/latest/download/install.sh | bash`,
} as const;

export const designDoc = (file: string): string => `${repo}/blob/main/archi/base/${file}`;

export const usualSetup = [
  "Admin panel on a public port",
  "Web UI and Docker socket in one process",
  "SSH open to the internet",
  "Panel compromised → root on the host",
];

export const withBrume = [
  "No inbound port by default: the firewall stays closed",
  "Panel on your tailnet by default, not on the internet",
  "Only the agent holds the Docker socket, and it listens on nothing",
  "Panel compromised → nothing without your session",
];

export interface Feature {
  readonly label: string;
  readonly title: string;
  readonly text: string;
}

export const features: readonly Feature[] = [
  {
    label: "Containers",
    title: "Any container",
    text: "Bring an image or a repository. Images are pinned by digest, so a moved tag never changes production behind your back.",
  },
  {
    label: "Static sites",
    title: "Instant rollback",
    text: "Served by a hardened, read-only nginx. Recent versions stay on disk, so going back is instant.",
  },
  {
    label: "Databases",
    title: "PostgreSQL, MongoDB, Redis",
    text: "Strong generated credentials, and an app account with only the rights it needs. Backups are on by default.",
  },
  {
    label: "CI/CD",
    title: "Deploy on push",
    text: "Push to a branch: Brume builds with Railpack and switches traffic once the new container is healthy. No pipeline to write.",
  },
  {
    label: "Backups",
    title: "Encrypted off the box",
    text: "Consistent dumps, encrypted with age and sent to any S3-compatible storage. Your server can encrypt them, never read them.",
  },
  {
    label: "Recovery",
    title: "Lose the server, not the state",
    text: "Brume streams its whole state to your bucket, encrypted. A lost VPS comes back from S3 with your age key, and every database dump is test-restored first.",
  },
  {
    label: "Logs",
    title: "Logs and alerts",
    text: "Logs from every container, searchable from the panel. Alerts by email, webhook or ntfy when something fails.",
  },
  {
    label: "Secrets",
    title: "Secrets stay secret",
    text: "Encrypted at rest, decrypted only by the engine and injected when the container starts. The panel shows one only when you ask to reveal it.",
  },
];

export const principles: readonly Feature[] = [
  {
    label: "Network",
    title: "Zero inbound ports",
    text: "By default, public traffic reaches your apps through a Cloudflare Tunnel, and the firewall accepts nothing from the internet, SSH included.",
  },
  {
    label: "Admin",
    title: "Panel on your tailnet",
    text: "By default, the panel is served on your tailnet only. Logging in takes your Tailscale identity and a second factor. Other modes are chosen on the host, never from the panel.",
  },
  {
    label: "Engine",
    title: "The keys, without Docker",
    text: "The engine holds the state and the secrets, and checks every session itself. It listens on unix sockets only, and never touches Docker.",
  },
  {
    label: "Agent",
    title: "Docker, behind a policy",
    text: "Only the agent holds the Docker socket, and it listens on nothing. It builds every container from an allowlist, and enforces the policy even against a compromised engine.",
  },
  {
    label: "Updates",
    title: "Signed releases",
    text: "The installer and every update are verified with minisign. The updater refuses to go back to an older version.",
  },
  {
    label: "Audit",
    title: "A log you can't erase",
    text: "Every action is an event, streamed to S3, encrypted and hash-chained. Brume can't delete from the bucket: an attacker can stop the log, not erase what already left.",
  },
];

export interface Comparison {
  readonly tool: string;
  readonly panel: string;
  readonly docker: string;
  readonly attackerGets: string;
}

/** From each tool's installation docs, see archi/base/11-comparatif.md. */
export const comparison: readonly Comparison[] = [
  {
    tool: "Portainer",
    panel: "Public, port 9443",
    docker: "Socket mounted in the UI container",
    attackerGets: "Root on the host",
  },
  {
    tool: "Coolify",
    panel: "Public, port 8000",
    docker: "Root SSH key stored in the app",
    attackerGets: "Root on every managed server",
  },
  {
    tool: "Dokploy",
    panel: "Public, port 3000",
    docker: "Socket mounted",
    attackerGets: "Root on the host",
  },
  { tool: "CapRover", panel: "Public", docker: "Socket mounted", attackerGets: "Root on the host" },
];

export const brumeComparison: Comparison = {
  tool: "Brume",
  panel: "Tailnet, by default",
  docker: "Separate agent, listens on nothing",
  attackerGets: "Nothing without your session",
};

export const designDocs = [
  { file: "01-modele-de-menace.md", topic: "threat model" },
  { file: "02-architecture.md", topic: "processes, privileges" },
  { file: "03-reseau.md", topic: "networking" },
  { file: "04-panel-admin.md", topic: "admin panel, auth" },
  { file: "05-runtime.md", topic: "container isolation" },
  { file: "06-builds-et-deploiement.md", topic: "builds, deploys" },
  { file: "07-secrets-et-backups.md", topic: "secrets, backups" },
  { file: "08-hote-et-installation.md", topic: "host setup" },
  { file: "09-observabilite.md", topic: "logs, audit, alerts" },
  { file: "13-modele-de-donnees.md", topic: "data model" },
  { file: "14-architecture-logicielle.md", topic: "engine, agent, event log" },
  { file: "15-etat-et-restauration.md", topic: "state on S3, restore" },
  { file: "16-tests.md", topic: "simulation, real VMs" },
  { file: "17-telemetrie.md", topic: "OpenTelemetry" },
];

export const stack = [
  "Rust",
  "tokio",
  "SQLite",
  "DuckDB",
  "S3",
  "OpenTelemetry",
  "Docker",
  "Traefik",
  "Cloudflare Tunnel",
  "Tailscale",
  "React",
];

export const roadmap = {
  shipping: [
    "One server, one owner",
    "One-command install",
    "Any container, static sites with instant rollback",
    "Templates: static site, PostgreSQL, MongoDB, Redis",
    "Deploy on push from GitHub, built with Railpack",
    "Backups, logs and secrets from the panel",
    "All of Brume's state on S3, encrypted: a lost server comes back from the bucket",
    "Panel over Tailscale, an SSH tunnel or in public",
    "Public traffic through Cloudflare Tunnel, or directly on ports 80 and 443",
  ],
  later: [
    "Several servers, each agent connecting out to the engine",
    "Pull request previews",
    "A web shell into containers",
    "API tokens and a remote CLI",
    "Point-in-time recovery for PostgreSQL",
    "A highly available engine",
    "Tailnet Lock and Headscale",
  ],
  requirements: [
    "A fresh VPS with Debian 12 or Ubuntu 24.04",
    "An S3-compatible bucket, with versioning or Object Lock",
    "Recommended: a domain on Cloudflare and a Tailscale account",
  ],
};
