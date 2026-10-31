import type { Edge, Node, XYPosition } from "@xyflow/react";

import type { Service } from "~/api/types";

export const NODE_WIDTH = 288;
// The service panel: `max-w-4xl` plus its `right-2` gap.
export const PANEL_WIDTH = 896 + 8;
const DEFAULT_NODE_HEIGHT = 120;
const COLUMN_GAP = 48;
const ROW_HEIGHT = 240;

export interface SavedPosition {
  readonly id: string;
  readonly x: number;
  readonly y: number;
}

// Rows from top to bottom: a service sits one row above the services it depends on,
// so apps come first and their databases below, like on Railway.
export function layoutServices(services: readonly Service[]): Map<string, XYPosition> {
  const ids = new Set<string>(services.map((service) => service.id));
  const dependents = new Map<string, string[]>();
  for (const service of services) {
    for (const dependency of service.dependencies) {
      if (ids.has(dependency)) {
        dependents.set(dependency, [...(dependents.get(dependency) ?? []), service.id]);
      }
    }
  }

  const rows = new Map<string, number>();
  const rowOf = (id: string, visiting: ReadonlySet<string>): number => {
    const known = rows.get(id);
    if (known !== undefined) {
      return known;
    }
    if (visiting.has(id)) {
      return 0;
    }
    const above = (dependents.get(id) ?? []).map((parent) =>
      rowOf(parent, new Set([...visiting, id])),
    );
    const row = above.length > 0 ? Math.max(...above) + 1 : 0;
    rows.set(id, row);
    return row;
  };

  const byRow = new Map<number, Service[]>();
  for (const service of services.toSorted((a, b) => a.name.localeCompare(b.name))) {
    const row = rowOf(service.id, new Set());
    byRow.set(row, [...(byRow.get(row) ?? []), service]);
  }

  const layout = new Map<string, XYPosition>();
  for (const [row, members] of byRow) {
    members.forEach((service, index) => {
      layout.set(service.id, {
        x: (index - (members.length - 1) / 2) * (NODE_WIDTH + COLUMN_GAP),
        y: row * ROW_HEIGHT,
      });
    });
  }
  return layout;
}

export function canvasNodes(services: readonly Service[], saved: readonly SavedPosition[]): Node[] {
  const layout = layoutServices(services);
  const savedById = new Map(saved.map((position) => [position.id, position]));
  return services.map((service) => {
    const position = savedById.get(service.id);
    return {
      id: service.id,
      type: "service",
      data: {},
      position:
        position && Number.isFinite(position.x) && Number.isFinite(position.y)
          ? { x: position.x, y: position.y }
          : (layout.get(service.id) ?? { x: 0, y: 0 }),
    };
  });
}

export function canvasEdges(services: readonly Service[]): Edge[] {
  const ids = new Set<string>(services.map((service) => service.id));
  return services.flatMap((service) =>
    service.dependencies
      .filter((dependency) => ids.has(dependency))
      .map((dependency) => ({
        id: `${service.id}->${dependency}`,
        source: service.id,
        target: dependency,
        type: "smoothstep",
      })),
  );
}

// Identifies the shape of the graph: the canvas starts over when it changes.
export function canvasKey(services: readonly Service[]): string {
  return services
    .map((service) => `${service.id}:${service.dependencies.toSorted().join(",")}`)
    .toSorted()
    .join("|");
}

interface FocusedNode {
  readonly position: XYPosition;
  readonly width?: number | undefined;
  readonly height?: number | undefined;
}

// Where to center the viewport so the selected node sits in the middle of the strip the panel
// leaves visible.
export function panelFocusCenter(
  node: FocusedNode,
  viewportWidth: number,
  zoom: number,
): XYPosition {
  const width = node.width ?? NODE_WIDTH;
  const center = {
    x: node.position.x + width / 2,
    y: node.position.y + (node.height ?? DEFAULT_NODE_HEIGHT) / 2,
  };
  const visible = viewportWidth - PANEL_WIDTH;
  if (visible < width * zoom) {
    return center;
  }
  return { x: center.x + (viewportWidth - visible) / 2 / zoom, y: center.y };
}
