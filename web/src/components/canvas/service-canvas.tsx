import "@xyflow/react/dist/style.css";

import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useNavigate } from "@tanstack/react-router";
import { Background, BackgroundVariant, ReactFlow } from "@xyflow/react";
import { BoxesIcon } from "lucide-react";

import { allNodePositions, saveNodePosition } from "~/api/canvas";
import { environmentServices } from "~/api/services";
import type { EnvironmentId, ProjectId } from "~/api/types";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";
import { canvasEdges, canvasKey, canvasNodes } from "~/helpers/canvas";
import { parseServiceId } from "~/helpers/services";

import { CanvasControls } from "./canvas-controls";
import { FocusSelectedService } from "./focus-selected-service";
import { ServiceNode } from "./service-node";

const NODE_TYPES = { service: ServiceNode };

// React Flow owns the node state (positions, measures, selection); the graph restarts only when its
// shape changes.
interface ServiceCanvasProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly environmentId: EnvironmentId;
}

export function ServiceCanvas({ projectId, environmentName, environmentId }: ServiceCanvasProps) {
  const navigate = useNavigate();
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
      onNodeClick={(_event, node) => {
        const serviceId = parseServiceId(node.id);
        if (serviceId) {
          void navigate({
            to: "/projects/$projectId/$env/services/$serviceId/deployments",
            params: { projectId, env: environmentName, serviceId },
          });
        }
      }}
    >
      <Background variant={BackgroundVariant.Dots} gap={16} />
      <CanvasControls services={services} />
      <FocusSelectedService />
    </ReactFlow>
  );
}
