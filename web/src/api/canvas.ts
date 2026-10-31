import {
  type InitialQueryBuilder,
  createCollection,
  localStorageCollectionOptions,
} from "@tanstack/react-db";
import type { XYPosition } from "@xyflow/react";

import type { SavedPosition } from "~/helpers/canvas";

// Where the admin dragged each service on the canvas. A layout preference of this browser,
// not project data: it stays in localStorage.
const nodePositionsCollection = createCollection(
  localStorageCollectionOptions<SavedPosition, string>({
    id: "canvas-positions",
    storageKey: "brume:canvas-positions",
    getKey: (position) => position.id,
  }),
);

export const allNodePositions = (q: InitialQueryBuilder) =>
  q.from({ position: nodePositionsCollection });

export function saveNodePosition(serviceId: string, { x, y }: XYPosition): void {
  if (nodePositionsCollection.has(serviceId)) {
    nodePositionsCollection.update(serviceId, (draft) => {
      draft.x = x;
      draft.y = y;
    });
  } else {
    nodePositionsCollection.insert({ id: serviceId, x, y });
  }
}

export function forgetNodePositions(serviceIds: readonly string[]): void {
  const saved = serviceIds.filter((id) => nodePositionsCollection.has(id));
  if (saved.length > 0) {
    nodePositionsCollection.delete(saved);
  }
}
