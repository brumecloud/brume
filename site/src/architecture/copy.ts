// The architecture page's headings, shared with its Markdown version.

export const architectureCopy = {
  title: "How Brume works.",
  lede: "Every part of Brume, why it's there, and what an attacker gets if they break it. Click any piece of the diagram, or follow a tour.",
  badge: "The design of alpha 0.1",
  modes:
    "Shown in the recommended modes: Cloudflare Tunnel for public traffic, Tailscale for administration. The other modes are chosen on the host, and are less safe.",
  explorerTitle: "The whole server, piece by piece",
  toursLabel: "Follow a tour",
  panelHintTitle: "Pick a component",
  panelHint:
    "Click any box to see why it's there, how it works, the rules it follows, and what an attacker gets if they break it.",
  principlesTitle: "The ideas behind it",
  principles: [
    "The public path and the admin path never meet.",
    "The engine decides. The agent acts.",
    "Only the agent holds Docker, and it listens on nothing.",
    "One system user and one socket per service.",
    "Brume can't delete what it sent to S3.",
    "Signed updates, never backwards.",
  ],
  componentsTitle: "Every component",
} as const;
