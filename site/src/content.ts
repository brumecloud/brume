// Links and copy shared across the landing page.

import { ActivityIcon } from "~/components/animate-ui/icons/activity";
import { BadgeCheckIcon } from "~/components/animate-ui/icons/badge-check";
import { BlocksIcon } from "~/components/animate-ui/icons/blocks";
import { CloudUploadIcon } from "~/components/animate-ui/icons/cloud-upload";
import { CogIcon } from "~/components/animate-ui/icons/cog";
import { FingerprintIcon } from "~/components/animate-ui/icons/fingerprint";
import { KeyRoundIcon } from "~/components/animate-ui/icons/key-round";
import { LayersIcon } from "~/components/animate-ui/icons/layers";
import { LockKeyholeIcon } from "~/components/animate-ui/icons/lock-keyhole";
import { RocketIcon } from "~/components/animate-ui/icons/rocket";
import { RotateCcwIcon } from "~/components/animate-ui/icons/rotate-ccw";
import { ScrollIcon } from "~/components/animate-ui/icons/scroll";
import { ShieldCheckIcon } from "~/components/animate-ui/icons/shield-check";
import { UnplugIcon } from "~/components/animate-ui/icons/unplug";

import type { BrandIconName } from "~/components/brand-icons";
import type { AnimatedIcon } from "~/components/icon-tile";
import type { Tone } from "~/components/tone";

const repo = "https://github.com/brumecloud/brume";

export const site = {
  name: "Brume",
  url: "https://brume.dev",
  title: "Brume · just push it",
  description:
    "Brume is an open source, self-hosted platform to deploy containers, static sites and databases on your own server, with zero inbound ports.",
  repo,
  issues: `${repo}/issues`,
  installCommand: "curl -sSL https://brume.dev/install.sh | bash",
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
  readonly icon: AnimatedIcon;
  readonly tone: Tone;
  readonly logos?: readonly BrandIconName[];
}

export const features: readonly Feature[] = [
  {
    label: "Containers",
    title: "Any container",
    text: "Bring an image or a repository. Images are pinned by digest, so a moved tag never changes production behind your back.",
    icon: BlocksIcon,
    tone: "sky",
  },
  {
    label: "Static sites",
    title: "Instant rollback",
    text: "Served by a hardened, read-only nginx. Recent versions stay on disk, so going back is instant.",
    icon: RotateCcwIcon,
    tone: "amber",
  },
  {
    label: "Databases",
    title: "PostgreSQL, MongoDB, Redis",
    text: "Strong generated credentials, and an app account with only the rights it needs. Backups are on by default.",
    icon: LayersIcon,
    tone: "emerald",
    logos: ["postgresql", "mongodb", "redis"],
  },
  {
    label: "CI/CD",
    title: "Deploy on push",
    text: "Push to a branch: Brume builds with Railpack and switches traffic once the new container is healthy. No pipeline to write.",
    icon: RocketIcon,
    tone: "violet",
  },
  {
    label: "Backups",
    title: "Encrypted off the box",
    text: "Consistent dumps, encrypted with age and sent to any S3-compatible storage. Your server can encrypt them, never read them.",
    icon: LockKeyholeIcon,
    tone: "teal",
  },
  {
    label: "Recovery",
    title: "Lose the server, not the state",
    text: "Brume streams its whole state to your bucket, encrypted. A lost VPS comes back from S3 with your age key, and every database dump is test-restored first.",
    icon: CloudUploadIcon,
    tone: "rose",
  },
  {
    label: "Logs",
    title: "Logs and alerts",
    text: "Logs from every container, searchable from the panel. Alerts by email, webhook or ntfy when something fails.",
    icon: ActivityIcon,
    tone: "orange",
  },
  {
    label: "Secrets",
    title: "Secrets stay secret",
    text: "Encrypted at rest, decrypted only by the engine and injected when the container starts. The panel shows one only when you ask to reveal it.",
    icon: KeyRoundIcon,
    tone: "fuchsia",
  },
];

export const principles: readonly Feature[] = [
  {
    label: "Network",
    title: "Zero inbound ports",
    text: "By default, public traffic reaches your apps through a Cloudflare Tunnel, and the firewall accepts nothing from the internet, SSH included.",
    icon: UnplugIcon,
    tone: "emerald",
  },
  {
    label: "Admin",
    title: "Panel on your tailnet",
    text: "By default, the panel is served on your tailnet only. Logging in takes your Tailscale identity and a second factor. Other modes are chosen on the host, never from the panel.",
    icon: FingerprintIcon,
    tone: "sky",
  },
  {
    label: "Engine",
    title: "The keys, without Docker",
    text: "The engine holds the state and the secrets, and checks every session itself. It listens on unix sockets only, and never touches Docker.",
    icon: CogIcon,
    tone: "violet",
  },
  {
    label: "Agent",
    title: "Docker, behind a policy",
    text: "Only the agent holds the Docker socket, and it listens on nothing. It builds every container from an allowlist, and enforces the policy even against a compromised engine.",
    icon: ShieldCheckIcon,
    tone: "teal",
  },
  {
    label: "Updates",
    title: "Signed releases",
    text: "The installer and every update are verified with minisign. The updater refuses to go back to an older version.",
    icon: BadgeCheckIcon,
    tone: "amber",
  },
  {
    label: "Audit",
    title: "A log you can't erase",
    text: "Every action is an event, streamed to S3, encrypted and hash-chained. Brume can't delete from the bucket: an attacker can stop the log, not erase what already left.",
    icon: ScrollIcon,
    tone: "rose",
  },
];

export interface Comparison {
  readonly tool: string;
  readonly icon?: BrandIconName;
  readonly panel: string;
  readonly docker: string;
  readonly attackerGets: string;
}

/** From each tool's installation docs, see archi/base/11-comparatif.md. */
export const comparison: readonly Comparison[] = [
  {
    tool: "Portainer",
    icon: "portainer",
    panel: "Public, port 9443",
    docker: "Socket mounted in the UI container",
    attackerGets: "Root on the host",
  },
  {
    tool: "Coolify",
    icon: "coolify",
    panel: "Public, port 8000",
    docker: "Root SSH key stored in the app",
    attackerGets: "Root on every managed server",
  },
  {
    tool: "Dokploy",
    icon: "dokploy",
    panel: "Public, port 3000",
    docker: "Socket mounted",
    attackerGets: "Root on the host",
  },
  {
    tool: "CapRover",
    icon: "caprover",
    panel: "Public",
    docker: "Socket mounted",
    attackerGets: "Root on the host",
  },
];

export const brumeComparison: Comparison = {
  tool: "Brume",
  panel: "Tailnet, by default",
  docker: "Separate agent, listens on nothing",
  attackerGets: "Nothing without your session",
};

export type MilestoneStatus = "current" | "next" | "later" | "ongoing";

export interface Milestone {
  readonly when: string;
  readonly status: MilestoneStatus;
  readonly title: string;
  readonly items: readonly string[];
}

/** The order after 0.1 is a direction, not a promise: see archi/base/12-plus-tard.md. */
export const milestones: readonly Milestone[] = [
  {
    when: "Now · in progress",
    status: "current",
    title: "Alpha 0.1",
    items: [
      "One server, one owner",
      "One-command install",
      "Any container, static sites with instant rollback",
      "Templates: static site, PostgreSQL, MongoDB, Redis",
      "Deploy on push from GitHub, built with Railpack",
      "Backups, logs and secrets from the panel",
      "All of Brume's state on S3, encrypted",
      "Panel over Tailscale, an SSH tunnel or in public",
      "Public traffic through Cloudflare Tunnel, or on ports 80 and 443",
    ],
  },
  {
    when: "Next",
    status: "next",
    title: "Developer workflow",
    items: ["Pull request previews", "A web shell into containers", "API tokens and a remote CLI"],
  },
  {
    when: "Later",
    status: "later",
    title: "More servers",
    items: [
      "Several servers, each agent connecting out to the engine",
      "A highly available engine",
    ],
  },
  {
    when: "All along",
    status: "ongoing",
    title: "Hardening",
    items: [
      "Tailnet Lock and Headscale",
      "Builds in a throwaway gVisor sandbox",
      "Rootless Docker and gVisor for apps",
      "Secrets as files on tmpfs",
    ],
  },
];

export const requirements =
  "Needs a fresh Debian 12 or Ubuntu 24.04 VPS and an S3-compatible bucket with versioning or Object Lock. Cloudflare and Tailscale are recommended.";

export interface StackItem {
  readonly icon: BrandIconName;
  readonly name: string;
}

/** What people bring: anything that builds to a container runs on Brume. */
export const stackItems: readonly StackItem[] = [
  { icon: "node", name: "Node.js" },
  { icon: "bun", name: "Bun" },
  { icon: "deno", name: "Deno" },
  { icon: "python", name: "Python" },
  { icon: "go", name: "Go" },
  { icon: "rust", name: "Rust" },
  { icon: "ruby", name: "Ruby" },
  { icon: "php", name: "PHP" },
  { icon: "java", name: "Java" },
  { icon: "elixir", name: "Elixir" },
  { icon: "nextjs", name: "Next.js" },
  { icon: "astro", name: "Astro" },
  { icon: "vite", name: "Vite" },
  { icon: "svelte", name: "Svelte" },
  { icon: "laravel", name: "Laravel" },
  { icon: "django", name: "Django" },
  { icon: "rails", name: "Rails" },
  { icon: "postgresql", name: "PostgreSQL" },
  { icon: "mongodb", name: "MongoDB" },
  { icon: "redis", name: "Redis" },
  { icon: "docker", name: "Docker" },
];
