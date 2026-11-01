// The roadmap page's data, shared with its Markdown version. Every item comes from
// docs/content/docs/how-it-works/roadmap.mdx (archi/base/12-plus-tard.md in the design notes).

import { type MilestoneStatus, milestones } from "~/content";

/** Where the landing's timeline already places an item. Never set from anywhere else. */
export type RoadmapWhen = Exclude<MilestoneStatus, "current">;

export interface RoadmapItem {
  readonly title: string;
  /** What the item brings, in one sentence of about twenty words at most. */
  readonly brings: string;
  /** Only where `milestones` in content.ts already lists the item. */
  readonly when?: RoadmapWhen;
}

export interface RoadmapTheme {
  readonly title: string;
  readonly items: readonly RoadmapItem[];
}

export interface OpenDecision {
  readonly title: string;
  readonly text: string;
}

export const roadmapCopy = {
  badge: "Alpha 0.1 · in progress",
  title: "What comes next.",
  lede: "What alpha 0.1 leaves out on purpose, and what each piece brings. The order after 0.1 is a direction, not a promise.",
  timelineTitle: "At a glance",
  themesTitle: "Everything planned.",
  themesLede: "A badge places an item on the timeline above. The others are not placed yet.",
  openTitle: "Still open.",
  openLede:
    "Design decisions not made yet. Until one is settled, the docs describe the options, not an answer.",
  docsLink: "The roadmap in the docs",
} as const;

/** The badge of an item: the `when` of its milestone, such as "Next" or "All along". */
export function whenLabel(when: RoadmapWhen): string {
  return milestones.find((milestone) => milestone.status === when)?.when ?? when;
}

export const roadmapThemes: readonly RoadmapTheme[] = [
  {
    title: "Access and isolation",
    items: [
      {
        title: "Tailnet Lock",
        brings:
          "Neither Tailscale's coordination server nor an account stolen from your identity provider can add a device to your tailnet.",
        when: "ongoing",
      },
      {
        title: "Disposable gVisor builder, or builds off the server",
        brings:
          "A build can no longer reach root through a kernel or runc flaw, or tamper with later builds.",
        when: "ongoing",
      },
      {
        title: "Secrets as files on tmpfs",
        brings:
          "An option: files don't leak to child processes, error reports or /proc, as environment variables do.",
      },
      {
        title: "Rootless Docker, gVisor for apps",
        brings: "Stronger isolation of the host and the containers, beyond userns-remap.",
        when: "ongoing",
      },
      {
        title: "systemd-analyze security in CI",
        brings: "Scores the exposure of each systemd unit. CI fails if the score gets worse.",
      },
    ],
  },
  {
    title: "Data and backups",
    items: [
      {
        title: "Hardened backups",
        brings:
          "Object Lock retention headers on every upload, so a bucket without Object Lock fails instead of accepting it.",
      },
      {
        title: "Master key rotation",
        brings: "New blobs on S3 use the new key. Old blobs stay readable through their key ID.",
      },
      {
        title: "Authenticated events on S3",
        brings:
          "Signed or encrypted log segments: whoever holds the S3 credential can no longer write fake events a restore would replay.",
      },
      {
        title: "Built images on S3",
        brings:
          "Restores with the exact images and no rebuilds, and rollbacks past the last 10 revisions.",
      },
      {
        title: "Backups of volumes outside templates",
        brings:
          "Backups for every volume. In 0.1, only the databases created from templates are backed up.",
      },
      {
        title: "Major database upgrades",
        brings:
          "Backup, dump, then restore into the new version. The engine then refuses a rollback to the old version.",
      },
      {
        title: "Database replicas and point-in-time recovery",
        brings:
          "Backups are dumps: writes since the last one can be lost. Postgres WAL archiving cuts that to a few seconds.",
      },
    ],
  },
  {
    title: "Detection and recovery",
    items: [
      {
        title: "External heartbeat",
        brings:
          "An outside service alerts you when Brume goes silent: a dead server, an attacker with root, or stopped S3 uploads.",
      },
      {
        title: "Point-in-time restore, and recovery after a compromise",
        brings:
          "Restore up to a date, an audit view from a date, and which secrets to revoke, in what order.",
      },
    ],
  },
  {
    title: "Features",
    items: [
      {
        title: "Web shell into containers",
        brings:
          "A shell into your containers, from the panel. Its most powerful feature, so it needs careful design.",
        when: "next",
      },
      {
        title: "Tokens for CI, deploys from GitHub Actions",
        brings:
          "A CI runner has no tailnet identity, so it needs its own access from outside Tailscale, with its own rules.",
        when: "next",
      },
      {
        title: "Remote MCP server",
        brings:
          "Served by brume api, for AI agents that can't run a local command. It needs OAuth.",
      },
      {
        title: "PR previews",
        brings:
          "A preview of each pull request. Already decided: a pull request from a fork is never built with secrets.",
        when: "next",
      },
      {
        title: "Outbound internet filtering by domain",
        brings:
          "Let a service reach only the domains it needs, such as api.stripe.com. It needs an egress proxy.",
      },
    ],
  },
  {
    title: "Hosting",
    items: [
      {
        title: "Multi-server",
        brings:
          "Agents on other machines connect outbound to the master, over the tailnet. Never a central root SSH key.",
        when: "later",
      },
      {
        title: "A highly available engine",
        brings:
          "The log is replicated with Raft, so the control plane is no longer a single point of failure.",
        when: "later",
      },
      {
        title: "Agent start without the engine",
        brings:
          "The agent restarts your apps at boot from its last desired state, even if the engine doesn't start.",
      },
      {
        title: "Red Hat family (Alma, Rocky, RHEL)",
        brings:
          "SELinux instead of AppArmor, dnf instead of apt. Alpha 0.1 runs on Debian and Ubuntu.",
      },
      {
        title: "Headscale",
        brings: "Full self-hosting of the admin layer. Leaving it out of 0.1 is still a proposal.",
        when: "ongoing",
      },
    ],
  },
];

export const openDecisions: readonly OpenDecision[] = [
  {
    title: "A contributor license agreement",
    text: "A CLA that keeps the option of changing the license, or none.",
  },
  {
    title: "Automatic updates",
    text: "Before the first release: when to install them, a fallback to the previous binary, a progressive rollout.",
  },
  {
    title: "Checking the signing key",
    text: "A way to verify it outside GitHub, since brume.dev deploys from the same repository.",
  },
  {
    title: "Local Cloudflare tunnel config",
    text: "Cloudflare's docs must confirm that it stops the dashboard from adding routes, and that the API can create it.",
  },
  {
    title: "Headscale",
    text: "Leaving it out of 0.1 is a proposal, to confirm later.",
  },
  {
    title: "A domain for your apps",
    text: "Apart from your email domain, so a stolen Cloudflare account can't redirect your MX records. Advice at install, or nothing.",
  },
  {
    title: "Panel frontend",
    text: "How the panel decodes API responses, and which library draws the metrics charts.",
  },
  {
    title: "After brume deploy from a folder",
    text: "Whether the service follows its branch on each push, or stays on the deployed commit.",
  },
];

/** The anchor of a theme on /roadmap, such as `theme-data-and-backups`. The footer links to it. */
export const themeId = (title: string) =>
  `theme-${title.toLowerCase().replaceAll(/[^a-z]+/g, "-")}`;
