import { archComponents } from "~/architecture/components";
import {
  COLUMNS,
  NODE_HEIGHT,
  NODE_WIDTH,
  ROWS,
  VIEW_HEIGHT,
  VIEW_WIDTH,
  archEdges,
  flowFillLit,
  flowStroke,
  flowStrokeLit,
} from "~/architecture/edges";
import type { ArchComponent, ComponentId, Flow } from "~/architecture/model";

import { SvgComponentIcon } from "./component-icon";

const flowFill: Record<Flow, string> = {
  public: "fill-orange-400",
  admin: "fill-sky-400",
  socket: "fill-violet-400",
  control: "fill-fuchsia-400",
  storage: "fill-emerald-400",
  docker: "fill-teal-400",
};

const percent = (value: number, total: number) => `${(value / total) * 100}%`;

/** The ID of a component's button, so the explorer can give it the focus back. */
export const nodeButtonId = (id: ComponentId) => `diagram-node-${id}`;

/** A box with pixel-art corners: one notch at each corner. */
function boxPath(x: number, y: number): string {
  const n = 3;
  const w = NODE_WIDTH;
  const h = NODE_HEIGHT;
  return `M${x + n} ${y}H${x + w - n}V${y + n}H${x + w}V${y + h - n}H${x + w - n}V${y + h}H${x + n}V${y + h - n}H${x}V${y + n}H${x + n}Z`;
}

function NodeBox({
  node,
  state,
}: {
  readonly node: ArchComponent;
  readonly state: "rest" | "selected" | "near" | "dim";
}) {
  const x = COLUMNS[node.column];
  const y = ROWS[node.row];
  const isBrume = node.kind === "brume";
  const box =
    state === "selected"
      ? "fill-highlight stroke-brand"
      : isBrume
        ? "fill-highlight stroke-ink-dim lit:stroke-brand"
        : "fill-surface stroke-edge-strong";
  return (
    <g
      className={`transition-opacity duration-300 ${state === "dim" ? "opacity-25" : "opacity-100"}`}
    >
      <path
        d={boxPath(x, y)}
        strokeWidth="2"
        shapeRendering="crispEdges"
        className={`transition duration-300 ${box}`}
      />
      <SvgComponentIcon icon={node.icon} size={22} x={x + 14} y={y + 17} />
      <text
        x={x + 48}
        y={y + 25}
        className={`text-[14px] font-medium fill-ink ${isBrume ? "transition duration-300 lit:fill-brand-soft" : ""}`}
      >
        {node.label}
      </text>
      <text x={x + 48} y={y + 42} className="fill-ink-dim text-[11.5px]">
        {node.detail}
      </text>
    </g>
  );
}

interface DiagramProps {
  /** The component whose panel is open, or the focus of the current tour step. */
  readonly selected: ComponentId | null;
  /** The links that keep their color while something is selected. */
  readonly litEdges: ReadonlySet<string>;
  readonly onSelect: (id: ComponentId) => void;
}

/**
 * The architecture drawn in SVG, with a real button over each component. At rest, it looks like
 * the landing's old diagram. Once a component is selected, everything not linked to it fades.
 */
export function Diagram({ selected, litEdges, onSelect }: DiagramProps) {
  const near = new Set<ComponentId>();
  for (const edge of archEdges) {
    if (litEdges.has(edge.id)) {
      near.add(edge.from);
      near.add(edge.to);
    }
  }
  const focused = selected !== null;
  const stateOf = (id: ComponentId) =>
    !focused ? "rest" : id === selected ? "selected" : near.has(id) ? "near" : "dim";

  return (
    <div className="relative min-w-[760px]">
      <svg
        aria-hidden="true"
        viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
        className="block w-full font-sans"
      >
        <defs>
          {(Object.keys(flowFill) as Flow[]).flatMap((flow) => [
            <marker
              key={flow}
              id={`arch-arrow-${flow}`}
              viewBox="0 0 8 8"
              refX="8"
              refY="4"
              markerWidth="8"
              markerHeight="8"
              markerUnits="userSpaceOnUse"
              orient="auto"
            >
              <path
                d="M0 0L8 4L0 8Z"
                className={`fill-ink-dim transition duration-300 ${flowFillLit[flow]}`}
              />
            </marker>,
            <marker
              key={`${flow}-on`}
              id={`arch-arrow-${flow}-on`}
              viewBox="0 0 8 8"
              refX="8"
              refY="4"
              markerWidth="8"
              markerHeight="8"
              markerUnits="userSpaceOnUse"
              orient="auto"
            >
              <path d="M0 0L8 4L0 8Z" className={flowFill[flow]} />
            </marker>,
          ])}
        </defs>
        {archEdges.map((edge) => {
          const points = edge.points.map(([x, y]) => `${x},${y}`).join(" ");
          const on = litEdges.has(edge.id);
          const faded = focused && !on;
          return (
            <g
              key={edge.id}
              className={`transition-opacity duration-300 ${faded ? "opacity-15" : "opacity-100"}`}
            >
              <polyline
                points={points}
                fill="none"
                strokeWidth="2"
                shapeRendering="crispEdges"
                markerEnd={`url(#arch-arrow-${edge.flow}${on ? "-on" : ""})`}
                className="stroke-edge-strong"
              />
              <polyline
                points={points}
                fill="none"
                strokeWidth="2"
                strokeDasharray="4 10"
                shapeRendering="crispEdges"
                className={`transition duration-300 ${
                  on
                    ? `flow ${flowStroke[edge.flow]}`
                    : faded
                      ? "stroke-ink-dim"
                      : `flow stroke-ink-dim ${flowStrokeLit[edge.flow]}`
                }`}
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
        {archComponents.map((node) => (
          <NodeBox key={node.id} node={node} state={stateOf(node.id)} />
        ))}
      </svg>
      {/* Real buttons over the boxes: keyboard, focus and screen readers come for free. */}
      {archComponents.map((node) => (
        <button
          key={node.id}
          id={nodeButtonId(node.id)}
          type="button"
          aria-pressed={selected === node.id}
          aria-label={`${node.label}, ${node.detail}`}
          onClick={() => onSelect(node.id)}
          className="absolute cursor-pointer rounded-sm transition-colors hover:bg-white/[0.04] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
          style={{
            left: percent(COLUMNS[node.column], VIEW_WIDTH),
            top: percent(ROWS[node.row], VIEW_HEIGHT),
            width: percent(NODE_WIDTH, VIEW_WIDTH),
            height: percent(NODE_HEIGHT, VIEW_HEIGHT),
          }}
        />
      ))}
    </div>
  );
}
