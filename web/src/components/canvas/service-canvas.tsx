import "@xyflow/react/dist/style.css";

import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Background, BackgroundVariant, ReactFlow } from "@xyflow/react";
import { BoxesIcon } from "lucide-react";

import { allNodePositions, saveNodePosition } from "~/api/canvas";
import { environmentServices } from "~/api/services";
import type { EnvironmentId } from "~/api/types";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";
import { canvasEdges, canvasKey, canvasNodes } from "~/helpers/canvas";

import { CanvasControls } from "./canvas-controls";
import { ServiceNode } from "./service-node";

const NODE_TYPES = { service: ServiceNode };

// Reference: infra/screens/03a-project-canvas-prod.png. React Flow owns the node state
// (positions, measures, selection); the graph restarts only when its shape changes.
export function ServiceCanvas({ environmentId }: { readonly environmentId: EnvironmentId }) {
  const { data: services } = useLiveSuspenseQuery({ query: environmentServices(environmentId) });
  const { data: positions } = useLiveSuspenseQuery({ query: allNodePositions });

  if (services.length === 0) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <BoxesIcon />
          </EmptyMedia>
          <EmptyTitle>Aucun service</EmptyTitle>
          <EmptyDescription>Cet environnement ne contient encore aucun service.</EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  return (
    <ReactFlow
      key={canvasKey(services)}
      defaultNodes={canvasNodes(services, positions)}
      defaultEdges={canvasEdges(services)}
      nodeTypes={NODE_TYPES}
      colorMode="dark"
      fitView
      fitViewOptions={{ maxZoom: 1 }}
      nodesConnectable={false}
      onNodeDragStop={(_event, node) => saveNodePosition(node.id, node.position)}
    >
      <Background variant={BackgroundVariant.Dots} gap={16} />
      <CanvasControls services={services} />
    </ReactFlow>
  );
}
