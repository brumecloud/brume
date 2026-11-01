// The diagram's grid and its links. The public path runs along the top, the admin path below
// it without crossing it, the engine sits in the middle, and the agent, Docker and the builder
// at the bottom.

import type { ArchEdge, Flow } from "./model";

export const NODE_WIDTH = 196;
export const NODE_HEIGHT = 56;
export const COLUMNS = [16, 276, 536, 796] as const;
export const ROWS = [16, 104, 192, 280, 368, 456] as const;
export const VIEW_WIDTH = COLUMNS[3] + NODE_WIDTH + 16;
export const VIEW_HEIGHT = ROWS[5] + NODE_HEIGHT + 16;

/** At rest, a link takes its color only while the diagram is lit. */
export const flowStrokeLit: Record<Flow, string> = {
  public: "lit:stroke-orange-400",
  admin: "lit:stroke-sky-400",
  socket: "lit:stroke-violet-400",
  control: "lit:stroke-fuchsia-400",
  storage: "lit:stroke-emerald-400",
  docker: "lit:stroke-teal-400",
};

export const flowFillLit: Record<Flow, string> = {
  public: "lit:fill-orange-400",
  admin: "lit:fill-sky-400",
  socket: "lit:fill-violet-400",
  control: "lit:fill-fuchsia-400",
  storage: "lit:fill-emerald-400",
  docker: "lit:fill-teal-400",
};

/** The links of the selected component, or of the current step of a tour, keep their color. */
export const flowStroke: Record<Flow, string> = {
  public: "stroke-orange-400",
  admin: "stroke-sky-400",
  socket: "stroke-violet-400",
  control: "stroke-fuchsia-400",
  storage: "stroke-emerald-400",
  docker: "stroke-teal-400",
};

const [c0, c1, c2, c3] = COLUMNS;
const [r0, r1, r2, r3, r4, r5] = ROWS;
const right = (column: number) => column + NODE_WIDTH;
const middle = (row: number) => row + NODE_HEIGHT / 2;
const center = (column: number) => column + NODE_WIDTH / 2;
const bottom = (row: number) => row + NODE_HEIGHT;

const edgeData = [
  // Public traffic.
  {
    id: "internet-cloudflare",
    from: "internet",
    to: "cloudflare",
    flow: "public",
    points: [
      [right(c0), middle(r0)],
      [c1, middle(r0)],
    ],
  },
  {
    id: "cloudflare-traefik",
    from: "cloudflare",
    to: "traefik",
    flow: "public",
    points: [
      [right(c1), middle(r0)],
      [c2, middle(r0)],
    ],
  },
  {
    id: "traefik-apps",
    from: "traefik",
    to: "apps",
    flow: "public",
    points: [
      [right(c2), middle(r0)],
      [c3, middle(r0)],
    ],
  },
  {
    id: "traefik-caddy",
    from: "traefik",
    to: "caddy",
    flow: "public",
    points: [
      [right(c2), r0 + 46],
      [right(c2) + 32, r0 + 46],
      [right(c2) + 32, middle(r1)],
      [c3, middle(r1)],
    ],
  },
  {
    id: "traefik-webhook",
    from: "traefik",
    to: "webhook",
    flow: "public",
    points: [
      [center(c2), bottom(r0)],
      [center(c2), r1],
    ],
  },
  // Administration.
  {
    id: "tailnet-api",
    from: "tailnet",
    to: "api",
    flow: "admin",
    points: [
      [right(c0), middle(r2)],
      [c1, middle(r2)],
    ],
  },
  // Both reach the engine over unix sockets.
  {
    id: "api-engine",
    from: "api",
    to: "engine",
    flow: "socket",
    points: [
      [right(c1), middle(r2)],
      [c2 + 54, middle(r2)],
      [c2 + 54, r3],
    ],
    label: { x: c2 + 62, y: r3 - 10, text: "unix sockets" },
  },
  {
    id: "webhook-engine",
    from: "webhook",
    to: "engine",
    flow: "socket",
    points: [
      [c2 + 140, bottom(r1)],
      [c2 + 140, r3],
    ],
  },
  // The engine, its bucket, its updater and the logs it relays.
  {
    id: "engine-s3",
    from: "engine",
    to: "s3",
    flow: "storage",
    points: [
      [c2, middle(r3)],
      [right(c0), middle(r3)],
    ],
    label: {
      x: (right(c0) + c2) / 2,
      y: middle(r3) - 8,
      text: "state, event log",
      anchor: "middle",
    },
  },
  {
    id: "engine-updater",
    from: "engine",
    to: "updater",
    flow: "control",
    points: [
      [c2 + 176, r3],
      [c2 + 176, middle(r2)],
      [c3, middle(r2)],
    ],
  },
  {
    id: "telemetry-engine",
    from: "telemetry",
    to: "engine",
    // brume telemetry has no network at all: the engine reads the logs over a unix socket.
    flow: "socket",
    points: [
      [c3, middle(r3)],
      [right(c2), middle(r3)],
    ],
  },
  // The engine decides, the agent acts.
  {
    id: "engine-agent",
    from: "engine",
    to: "agent",
    flow: "control",
    points: [
      [c2 + 60, bottom(r3)],
      [c2 + 60, r4],
    ],
    label: { x: c2 + 52, y: bottom(r3) + 21, text: "desired state", anchor: "end" },
  },
  {
    id: "agent-engine",
    from: "agent",
    to: "engine",
    flow: "control",
    points: [
      [c2 + 136, r4],
      [c2 + 136, bottom(r3)],
    ],
    label: { x: c2 + 144, y: bottom(r3) + 21, text: "facts" },
  },
  {
    id: "agent-docker",
    from: "agent",
    to: "docker",
    flow: "docker",
    points: [
      [right(c2), middle(r4)],
      [c3, middle(r4)],
    ],
  },
  {
    id: "docker-telemetry",
    from: "docker",
    to: "telemetry",
    flow: "docker",
    points: [
      [center(c3), r4],
      [center(c3), bottom(r3)],
    ],
    label: { x: center(c3) + 8, y: bottom(r3) + 21, text: "app logs" },
  },
  {
    id: "agent-s3",
    from: "agent",
    to: "s3",
    flow: "storage",
    points: [
      [c2, middle(r4)],
      [center(c0), middle(r4)],
      [center(c0), bottom(r3)],
    ],
    label: { x: (center(c0) + c2) / 2 + 48, y: middle(r4) - 8, text: "backups", anchor: "middle" },
  },
  {
    id: "agent-buildkit",
    from: "agent",
    to: "buildkit",
    flow: "docker",
    points: [
      [center(c2), bottom(r4)],
      [center(c2), r5],
    ],
  },
] as const satisfies readonly ArchEdge[];

/** The ID of a link that is drawn. Tours refer to links by these. */
export type EdgeId = (typeof edgeData)[number]["id"];

export const archEdges: readonly ArchEdge[] = edgeData;
