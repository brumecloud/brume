import { type BrandIconName, brandIcons } from "~/components/brand-icons";

// The diagram is drawn on a fixed grid: four columns of nodes, five rows.
const NODE_WIDTH = 196;
const NODE_HEIGHT = 56;
const COLUMNS = [16, 276, 536, 796] as const;
const ROWS = [16, 104, 192, 280, 368] as const;
const VIEW_WIDTH = COLUMNS[3] + NODE_WIDTH + 16;
const VIEW_HEIGHT = ROWS[4] + NODE_HEIGHT + 16;

type Glyph = "globe" | "storage" | "apps";

type NodeIcon =
  | { readonly kind: "brand"; readonly name: BrandIconName }
  | { readonly kind: "glyph"; readonly name: Glyph }
  | { readonly kind: "brume" };

interface DiagramNode {
  readonly column: 0 | 1 | 2 | 3;
  readonly row: 0 | 1 | 2 | 3 | 4;
  readonly label: string;
  readonly detail: string;
  readonly icon: NodeIcon;
}

const brume: NodeIcon = { kind: "brume" };

const nodes: readonly DiagramNode[] = [
  {
    column: 0,
    row: 0,
    label: "Internet",
    detail: "visitors and GitHub",
    icon: { kind: "glyph", name: "globe" },
  },
  {
    column: 1,
    row: 0,
    label: "Cloudflare Tunnel",
    detail: "no inbound port",
    icon: { kind: "brand", name: "cloudflare" },
  },
  {
    column: 2,
    row: 0,
    label: "Traefik",
    detail: "routes to your apps",
    icon: { kind: "brand", name: "traefik" },
  },
  {
    column: 3,
    row: 0,
    label: "Your apps",
    detail: "containers, sites",
    icon: { kind: "glyph", name: "apps" },
  },
  { column: 3, row: 1, label: "brume webhook", detail: "GitHub pushes", icon: brume },
  {
    column: 0,
    row: 2,
    label: "Your tailnet",
    detail: "Tailscale",
    icon: { kind: "brand", name: "tailscale" },
  },
  { column: 1, row: 2, label: "brume api", detail: "the panel", icon: brume },
  {
    column: 0,
    row: 3,
    label: "S3 bucket",
    detail: "any S3-compatible",
    icon: { kind: "glyph", name: "storage" },
  },
  { column: 2, row: 3, label: "brume engine", detail: "state and secrets", icon: brume },
  { column: 2, row: 4, label: "brume agent", detail: "the Docker socket", icon: brume },
  {
    column: 3,
    row: 4,
    label: "Docker",
    detail: "your containers",
    icon: { kind: "brand", name: "docker" },
  },
];

/** Each path gets its own color, with packets flowing along it. */
type Flow = "public" | "admin" | "socket" | "control" | "storage" | "docker";

const flowStroke: Record<Flow, string> = {
  public: "stroke-orange-400",
  admin: "stroke-sky-400",
  socket: "stroke-violet-400",
  control: "stroke-fuchsia-400",
  storage: "stroke-emerald-400",
  docker: "stroke-teal-400",
};

const flowFill: Record<Flow, string> = {
  public: "fill-orange-400",
  admin: "fill-sky-400",
  socket: "fill-violet-400",
  control: "fill-fuchsia-400",
  storage: "fill-emerald-400",
  docker: "fill-teal-400",
};

interface Edge {
  readonly flow: Flow;
  readonly points: readonly (readonly [number, number])[];
  readonly label?: {
    readonly x: number;
    readonly y: number;
    readonly text: string;
    readonly anchor?: "start" | "middle" | "end";
  };
}

const [c0, c1, c2, c3] = COLUMNS;
const [r0, r1, r2, r3, r4] = ROWS;
const right = (column: number) => column + NODE_WIDTH;
const middle = (row: number) => row + NODE_HEIGHT / 2;
const center = (column: number) => column + NODE_WIDTH / 2;
const bottom = (row: number) => row + NODE_HEIGHT;

const edges: readonly Edge[] = [
  // Public traffic.
  {
    flow: "public",
    points: [
      [right(c0), middle(r0)],
      [c1, middle(r0)],
    ],
  },
  {
    flow: "public",
    points: [
      [right(c1), middle(r0)],
      [c2, middle(r0)],
    ],
  },
  {
    flow: "public",
    points: [
      [right(c2), middle(r0)],
      [c3, middle(r0)],
    ],
  },
  {
    flow: "public",
    points: [
      [center(c2), bottom(r0)],
      [center(c2), middle(r1)],
      [c3, middle(r1)],
    ],
  },
  // Admin traffic.
  {
    flow: "admin",
    points: [
      [right(c0), middle(r2)],
      [c1, middle(r2)],
    ],
  },
  // Both reach the engine over unix sockets.
  {
    flow: "socket",
    points: [
      [right(c1), middle(r2)],
      [c2 + 54, middle(r2)],
      [c2 + 54, r3],
    ],
    label: { x: c2 + 62, y: r3 - 26, text: "unix socket" },
  },
  {
    flow: "socket",
    points: [
      [center(c3), bottom(r1)],
      [center(c3), middle(r3)],
      [right(c2), middle(r3)],
    ],
    label: { x: center(c3) + 8, y: r2 + 48, text: "unix socket" },
  },
  // The engine and the agent.
  {
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
    flow: "control",
    points: [
      [c2 + 60, bottom(r3)],
      [c2 + 60, r4],
    ],
    label: { x: c2 + 52, y: bottom(r3) + 21, text: "desired state", anchor: "end" },
  },
  {
    flow: "control",
    points: [
      [c2 + 136, r4],
      [c2 + 136, bottom(r3)],
    ],
    label: { x: c2 + 144, y: bottom(r3) + 21, text: "facts" },
  },
  {
    flow: "docker",
    points: [
      [right(c2), middle(r4)],
      [c3, middle(r4)],
    ],
  },
  {
    flow: "storage",
    points: [
      [c2, middle(r4)],
      [center(c0), middle(r4)],
      [center(c0), bottom(r3)],
    ],
    label: {
      x: (center(c0) + c2) / 2,
      y: middle(r4) - 8,
      text: "backups, builds",
      anchor: "middle",
    },
  },
];

/** A box with pixel-art corners: one notch at each corner. */
function boxPath(x: number, y: number): string {
  const n = 3;
  const w = NODE_WIDTH;
  const h = NODE_HEIGHT;
  return `M${x + n} ${y}H${x + w - n}V${y + n}H${x + w}V${y + h - n}H${x + w - n}V${y + h}H${x + n}V${y + h - n}H${x}V${y + n}H${x + n}Z`;
}

function Glyph({ name }: { readonly name: Glyph }) {
  switch (name) {
    case "globe":
      return (
        <>
          <circle cx="12" cy="12" r="9" />
          <path d="M3 12h18M12 3c3 3.2 3 14.8 0 18M12 3c-3 3.2-3 14.8 0 18" />
        </>
      );
    case "storage":
      return (
        <>
          <ellipse cx="12" cy="6" rx="8" ry="3" />
          <path d="M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3" />
        </>
      );
    case "apps":
      return (
        <>
          <rect x="4" y="4" width="7" height="7" />
          <rect x="13" y="4" width="7" height="7" />
          <rect x="4" y="13" width="7" height="7" />
          <rect x="13" y="13" width="7" height="7" />
        </>
      );
  }
}

function NodeIconMark({
  icon,
  x,
  y,
}: {
  readonly icon: NodeIcon;
  readonly x: number;
  readonly y: number;
}) {
  switch (icon.kind) {
    case "brume":
      return (
        <text x={x + 11} y={y + 18} textAnchor="middle" fontSize="18">
          ☁️
        </text>
      );
    case "brand": {
      const { path, color, viewBox, fillRule } = brandIcons[icon.name];
      return (
        <svg x={x} y={y} width="22" height="22" viewBox={viewBox}>
          <path d={path} fillRule={fillRule} style={{ fill: color }} />
        </svg>
      );
    }
    case "glyph":
      return (
        <svg
          x={x}
          y={y}
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          strokeWidth="1.75"
          strokeLinejoin="round"
          className="stroke-ink"
        >
          <Glyph name={icon.name} />
        </svg>
      );
  }
}

function DiagramNodeBox({ node }: { readonly node: DiagramNode }) {
  const x = COLUMNS[node.column];
  const y = ROWS[node.row];
  const isBrume = node.icon.kind === "brume";
  return (
    <g>
      <path
        d={boxPath(x, y)}
        strokeWidth="2"
        shapeRendering="crispEdges"
        className={isBrume ? "fill-highlight stroke-brand" : "fill-surface stroke-edge-strong"}
      />
      <NodeIconMark icon={node.icon} x={x + 14} y={y + 17} />
      <text
        x={x + 48}
        y={y + 25}
        className={`text-[14px] font-medium ${isBrume ? "fill-brand-soft" : "fill-ink"}`}
      >
        {node.label}
      </text>
      <text x={x + 48} y={y + 42} className="fill-ink-dim text-[11.5px]">
        {node.detail}
      </text>
    </g>
  );
}

export function ArchitectureDiagram() {
  return (
    <figure className="pixel-border reveal mt-12 bg-terminal">
      <div className="overflow-x-auto px-4 py-6 sm:px-6">
        <svg
          // oxlint-disable-next-line jsx-a11y/prefer-tag-over-role -- an inline SVG needs role="img" to be read as one image
          role="img"
          aria-labelledby="architecture-title"
          viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
          className="w-full min-w-[760px] font-sans"
        >
          <title id="architecture-title">
            Public traffic goes from the internet through a Cloudflare tunnel and Traefik to your
            apps and to the Brume webhook. Admin traffic goes from your tailnet to the Brume API.
            Both reach the engine over unix sockets. The engine keeps the state and the secrets,
            saves them to S3, and sends the desired state to the agent, the only component with the
            Docker socket.
          </title>
          <defs>
            {Object.entries(flowFill).map(([flow, fill]) => (
              <marker
                key={flow}
                id={`architecture-arrow-${flow}`}
                viewBox="0 0 8 8"
                refX="8"
                refY="4"
                markerWidth="8"
                markerHeight="8"
                markerUnits="userSpaceOnUse"
                orient="auto"
              >
                <path d="M0 0L8 4L0 8Z" className={fill} />
              </marker>
            ))}
          </defs>
          {edges.map((edge) => {
            const points = edge.points.map(([x, y]) => `${x},${y}`).join(" ");
            return (
              <g key={points}>
                <polyline
                  points={points}
                  fill="none"
                  strokeWidth="2"
                  shapeRendering="crispEdges"
                  markerEnd={`url(#architecture-arrow-${edge.flow})`}
                  className="stroke-edge-strong"
                />
                <polyline
                  points={points}
                  fill="none"
                  strokeWidth="2"
                  strokeDasharray="4 10"
                  shapeRendering="crispEdges"
                  className={`flow ${flowStroke[edge.flow]}`}
                />
                {edge.label ? (
                  <text
                    x={edge.label.x}
                    y={edge.label.y}
                    textAnchor={edge.label.anchor ?? "start"}
                    className="fill-ink-dim text-[11px]"
                  >
                    {edge.label.text}
                  </text>
                ) : null}
              </g>
            );
          })}
          {nodes.map((node) => (
            <DiagramNodeBox key={node.label} node={node} />
          ))}
        </svg>
      </div>
      <figcaption className="border-t-2 border-edge px-6 py-3.5 text-sm text-ink-muted">
        The public path and the admin path never meet. The engine never touches Docker, and the
        agent listens on nothing.
      </figcaption>
    </figure>
  );
}
