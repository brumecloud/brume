// Links and copy shared across the landing page.

import { ActivityIcon } from "~/components/animate-ui/icons/activity";
import { BadgeCheckIcon } from "~/components/animate-ui/icons/badge-check";
import { BlocksIcon } from "~/components/animate-ui/icons/blocks";
import { CloudUploadIcon } from "~/components/animate-ui/icons/cloud-upload";
import { FingerprintIcon } from "~/components/animate-ui/icons/fingerprint";
import { LayersIcon } from "~/components/animate-ui/icons/layers";
import { LockKeyholeIcon } from "~/components/animate-ui/icons/lock-keyhole";
import { RocketIcon } from "~/components/animate-ui/icons/rocket";
import { RotateCcwIcon } from "~/components/animate-ui/icons/rotate-ccw";
import { ShieldCheckIcon } from "~/components/animate-ui/icons/shield-check";
import { UnplugIcon } from "~/components/animate-ui/icons/unplug";

import type { BrandIconName } from "~/components/brand-icons";
import type { AnimatedIcon } from "~/components/icon-tile";
import type { Tone } from "~/components/tone";
import type { VsSlug } from "~/vs/model";

const repo = "https://github.com/brumecloud/brume";

export const site = {
  name: "Brume",
  url: "https://brume.dev",
  title: "Brume · just push it",
  description:
    "Brume is a free, open source platform to deploy anything on your own server: containers, static sites and databases, with zero open ports.",
  repo,
  issues: `${repo}/issues`,
  installCommand: "curl -sSL https://brume.dev/install.sh | bash",
} as const;

/** The sub-pages, in the order of the header. Each one also has a Markdown twin at `<path>/index.md`. */
export interface SitePage {
  readonly path: string;
  readonly title: string;
  /** The meta description, also used by /llms.txt. */
  readonly description: string;
}

export const pages: readonly SitePage[] = [
  {
    path: "/architecture",
    title: "Architecture",
    description:
      "How Brume is built: every component, why it's there, the security rules it follows, and what an attacker gets if they break it.",
  },
  {
    path: "/roadmap",
    title: "Roadmap",
    description:
      "What Brume's alpha 0.1 leaves out on purpose, what each planned piece brings, and the design decisions still open.",
  },
  {
    path: "/vs",
    title: "Compare",
    description:
      "Brume next to Coolify, Dokploy, Kamal, Railway, Render, Vercel and more: sourced facts, and where each one is better today.",
  },
];

export function pageAt(path: SitePage["path"]): SitePage {
  const found = pages.find((page) => page.path === path);
  if (!found) throw new Error(`No page at ${path} in content.ts`);
  return found;
}

/** A page of the public docs, in English, on GitHub until the docs site is online. */
export const docsPage = (path: string): string => `${repo}/blob/main/docs/content/docs/${path}.mdx`;

/** Headings and ledes of the page, shared with its Markdown version (`/index.md`). */
export const copy = {
  hero: {
    title: "just push it",
    lede: "Deploy anything on your own server, without the complexity. Your apps go online. Your server stays invisible.",
  },
  deploy: {
    title: "Deploy anything.",
    lede: "Push your code. Brume builds it, ships it and puts it online. No Dockerfile, no pipeline, no YAML.",
    stack: "If it builds into a container, it runs on Brume",
  },
  security: {
    title: "Secure by design.",
    lede: "Built on Cloudflare and Tailscale. Nothing on your server answers the internet.",
    comparisonTitle: "If the admin panel gets hacked…",
    comparisonSource: "From each project's install docs.",
    more: "Also: signed releases, secrets encrypted at rest, and an audit log you can't erase.",
  },
  console: {
    title: "Made for you, and your agents.",
    lede: "A panel that stays clear at 3 a.m. A CLI and an MCP server your agents can drive.",
    // Only on the page, which embeds the console.
    invitation: "The demo below is live. Click around.",
    agentTitle: "Your agent deploys. You approve.",
    agentLede:
      "Claude Code, Codex or Cursor ship with one sentence, through the same API as the panel.",
  },
  setup: {
    title: "One command. Nothing to babysit.",
    lede: "Brume updates itself, backs itself up, and comes back from your bucket if the server dies.",
  },
  roadmap: { title: "Alpha today. Here's what's next." },
  closing: { title: "Free. Open source. Yours.", lede: "One command on a fresh VPS." },
} as const;

/** Under the hero title: what Brume is, in four words. */
export const heroPoints = ["Free", "Open source", "Zero open ports", "Agent-ready"] as const;

export interface Feature {
  readonly label: string;
  readonly title: string;
  readonly text: string;
  readonly icon: AnimatedIcon;
  readonly tone: Tone;
  readonly logos?: readonly BrandIconName[];
}

export const deployFeatures: readonly Feature[] = [
  {
    label: "Containers",
    title: "Any container",
    text: "From a repo or an image, pinned by digest.",
    icon: BlocksIcon,
    tone: "sky",
  },
  {
    label: "Static sites",
    title: "Instant rollback",
    text: "Served by a hardened Caddy. Go back in a click.",
    icon: RotateCcwIcon,
    tone: "amber",
  },
  {
    label: "Databases",
    title: "PostgreSQL, MongoDB, Redis",
    text: "Strong credentials. Backups on by default.",
    icon: LayersIcon,
    tone: "emerald",
    logos: ["postgresql", "mongodb", "redis"],
  },
  {
    label: "CI/CD",
    title: "Deploy on push",
    text: "Traffic switches once the new version is healthy.",
    icon: RocketIcon,
    tone: "violet",
  },
];

export const securityClaims: readonly Feature[] = [
  {
    label: "Network",
    title: "Zero open ports.",
    text: "Traffic comes in through a Cloudflare Tunnel. The firewall stays shut, SSH included.",
    icon: UnplugIcon,
    tone: "emerald",
  },
  {
    label: "Admin",
    title: "No public panel.",
    text: "The panel lives on your tailnet, behind your Tailscale identity and a second factor.",
    icon: FingerprintIcon,
    tone: "sky",
  },
  {
    label: "Docker",
    title: "No root to steal.",
    text: "Only an isolated agent touches Docker, and it listens on nothing.",
    icon: ShieldCheckIcon,
    tone: "violet",
  },
];

export interface Comparison {
  readonly tool: string;
  /** Its page under /vs. */
  readonly slug: VsSlug;
  readonly icon?: BrandIconName;
  readonly panel: string;
  readonly attackerGets: string;
}

/** From each tool's installation docs, see docs/content/docs/how-it-works/comparison.mdx. */
export const comparison: readonly Comparison[] = [
  {
    tool: "Portainer",
    slug: "portainer",
    icon: "portainer",
    panel: "Public, port 9443",
    attackerGets: "Root on the host",
  },
  {
    tool: "Coolify",
    slug: "coolify",
    icon: "coolify",
    panel: "Public, port 8000",
    attackerGets: "Root on every managed server",
  },
  {
    tool: "Dokploy",
    slug: "dokploy",
    icon: "dokploy",
    panel: "Public, port 3000",
    attackerGets: "Root on the host",
  },
  {
    tool: "CapRover",
    slug: "caprover",
    icon: "caprover",
    panel: "Public",
    attackerGets: "Root on the host",
  },
];

export const brumeComparison: Omit<Comparison, "slug"> = {
  tool: "Brume",
  panel: "Your tailnet only",
  attackerGets: "Nothing without your session",
};

/** What an AI agent gets, and what it never does alone: see archi/base/19-cli-et-agents-ia.md. */
export const agentPoints = [
  "Deploy anything in one sentence",
  "Secrets never enter its context",
  "Anything irreversible waits for your approval",
] as const;

export type StepId = "install" | "github" | "push";

export interface Step {
  readonly id: StepId;
  readonly title: string;
  readonly text: string;
  // The title is a command to type.
  readonly command?: boolean;
}

export const steps: readonly Step[] = [
  {
    id: "install",
    title: "Install",
    text: "One command. Firewall, tailnet, tunnel and backups are set up for you.",
  },
  { id: "github", title: "Connect GitHub", text: "Install the GitHub App, pick a branch." },
  { id: "push", title: "git push", command: true, text: "You're live." },
];

export const setupFeatures: readonly Feature[] = [
  {
    label: "Updates",
    title: "Updates itself",
    text: "Every release is signed and checked before it runs.",
    icon: BadgeCheckIcon,
    tone: "amber",
  },
  {
    label: "Backups",
    title: "Encrypted off the box",
    text: "Database dumps, encrypted and sent to any S3.",
    icon: LockKeyholeIcon,
    tone: "teal",
  },
  {
    label: "Recovery",
    title: "Lose the server, not the state",
    text: "Restore it all on a new VPS, from your bucket.",
    icon: CloudUploadIcon,
    tone: "rose",
  },
  {
    label: "Logs",
    title: "Logs and alerts",
    text: "Search every log. Get alerted by email, webhook or ntfy.",
    icon: ActivityIcon,
    tone: "orange",
  },
];

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
      "Containers, static sites, databases",
      "Deploy on push from GitHub",
      "A CLI and an MCP server for agents",
      "Backups and full restore from S3",
    ],
  },
  {
    when: "Next",
    status: "next",
    title: "Developer workflow",
    items: ["Pull request previews", "A web shell into containers", "Deploys from CI"],
  },
  {
    when: "Later",
    status: "later",
    title: "More servers",
    items: ["Several servers, one panel", "A highly available engine"],
  },
  {
    when: "All along",
    status: "ongoing",
    title: "Hardening",
    items: ["Builds in a gVisor sandbox", "Rootless Docker", "Tailnet Lock and Headscale"],
  },
];

export const requirements =
  "Needs a fresh Debian or Ubuntu VPS and an S3-compatible bucket. Cloudflare and Tailscale recommended.";

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
