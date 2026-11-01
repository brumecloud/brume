// Tours walk through the diagram one step at a time. Each step lights a component, and the
// link it arrives by. The captions follow the docs: networking, builds-and-deployments,
// admin-panel, cli-and-ai-agents and secrets-and-backups.

import type { ArchEdge, ComponentId } from "./model";

export interface TourStep {
  readonly focus: ComponentId;
  readonly edge?: ArchEdge["id"];
  readonly caption: string;
}

export interface Tour {
  readonly id: "visit" | "push" | "command" | "state" | "restore";
  readonly title: string;
  readonly steps: readonly TourStep[];
}

export const tours: readonly Tour[] = [
  {
    id: "visit",
    title: "A visitor opens your app",
    steps: [
      {
        focus: "internet",
        caption: "Someone opens app.example.com. The DNS points to Cloudflare, not to your server.",
      },
      {
        focus: "cloudflare",
        edge: "internet-cloudflare",
        caption:
          "Cloudflare ends TLS and sends the request down the tunnel that cloudflared opened from your server.",
      },
      {
        focus: "traefik",
        edge: "cloudflare-traefik",
        caption: "Traefik matches the domain to a service. An unknown domain gets a 404.",
      },
      {
        focus: "apps",
        edge: "traefik-apps",
        caption: "The request reaches the app through a network it shares with Traefik alone.",
      },
      {
        focus: "caddy",
        edge: "traefik-caddy",
        caption:
          "A static site goes to Caddy instead, which serves its current revision from disk.",
      },
    ],
  },
  {
    id: "push",
    title: "You push to GitHub",
    steps: [
      {
        focus: "internet",
        caption: "You push to a configured branch. GitHub sends a signed webhook.",
      },
      {
        focus: "cloudflare",
        edge: "internet-cloudflare",
        caption: "It comes in through the tunnel, like any public request.",
      },
      {
        focus: "traefik",
        edge: "cloudflare-traefik",
        caption: "Traefik lets only GitHub's IP ranges reach the webhook, with a rate limit.",
      },
      {
        focus: "webhook",
        edge: "traefik-webhook",
        caption: "The webhook checks the HMAC signature, then asks the engine to queue a build.",
      },
      {
        focus: "engine",
        edge: "webhook-engine",
        caption: "The engine checks that the ref is the configured branch.",
      },
      {
        focus: "agent",
        edge: "engine-agent",
        caption:
          "It asks the agent for the build. The clone and the build run in the builder, never in the engine or the agent.",
      },
      {
        focus: "buildkit",
        edge: "agent-buildkit",
        caption:
          "The agent starts BuildKit, which clones the repo with a one-hour token and builds the image with Railpack.",
      },
      {
        focus: "docker",
        edge: "agent-docker",
        caption:
          "The agent loads the image into Docker, and the engine creates the revision and deploys it by digest.",
      },
      {
        focus: "apps",
        edge: "traefik-apps",
        caption:
          "Without a volume, the new version starts next to the old one, and traffic switches once its healthcheck passes. If it fails, the old version keeps running.",
      },
    ],
  },
  {
    id: "command",
    title: "You, or your AI agent, run a command",
    steps: [
      {
        focus: "tailnet",
        caption:
          "You click in the panel, or your AI agent calls the CLI. Both come in through your tailnet.",
      },
      {
        focus: "api",
        edge: "tailnet-api",
        caption:
          "tailscale serve adds your identity. The API forwards the request with your session or the CLI's signature.",
      },
      {
        focus: "engine",
        edge: "api-engine",
        caption:
          "The engine checks the session or the signature, then the policy. Anything irreversible waits for your passkey.",
      },
      {
        focus: "s3",
        edge: "engine-s3",
        caption: "The command becomes an event in the log, which leaves for your bucket.",
      },
      {
        focus: "agent",
        edge: "engine-agent",
        caption: "The engine sends the agent the new desired state of the machine.",
      },
      {
        focus: "docker",
        edge: "agent-docker",
        caption: "The agent checks it against its own policy, then drives Docker.",
      },
      {
        focus: "engine",
        edge: "agent-engine",
        caption: "The agent reports what it observed, and the engine records the facts.",
      },
    ],
  },
  {
    id: "state",
    title: "Brume saves its state",
    steps: [
      {
        focus: "engine",
        caption: "Every change is an event in the engine's log, in SQLite.",
      },
      {
        focus: "s3",
        edge: "engine-s3",
        caption:
          "The engine ships the log to your bucket in the background, secrets encrypted. An S3 outage blocks nothing.",
      },
      {
        focus: "agent",
        edge: "engine-agent",
        caption:
          "The agent makes a consistent dump of each database, and test-restores it before it leaves.",
      },
      {
        focus: "s3",
        edge: "agent-s3",
        caption: "The dump is encrypted for your age key, then uploaded. Brume can't delete it.",
      },
      {
        focus: "telemetry",
        edge: "docker-telemetry",
        caption: "App logs go to brume telemetry, never to the bucket: logs can hold secrets.",
      },
    ],
  },
  {
    id: "restore",
    title: "The server dies",
    steps: [
      {
        focus: "s3",
        caption:
          "Your server is gone. Your bucket still holds the log, the encrypted secrets and the dumps.",
      },
      {
        focus: "engine",
        edge: "engine-s3",
        caption:
          "On a new VPS, brume restore takes the bucket credentials and your age key, then replays the log and checks its hash chain.",
      },
      {
        focus: "cloudflare",
        caption:
          "It creates a new tunnel and repoints your domain, so the old server gets no traffic if it comes back.",
      },
      {
        focus: "buildkit",
        edge: "agent-buildkit",
        caption: "It rebuilds the images of the revisions in service from their commits.",
      },
      {
        focus: "s3",
        edge: "agent-s3",
        caption: "It restores the last dump of each database, and shows you how old it is.",
      },
      {
        focus: "apps",
        caption:
          "The agent starts everything. You sign in again: sessions don't survive a restore.",
      },
    ],
  },
];
