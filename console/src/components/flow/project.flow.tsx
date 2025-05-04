import AppGroupFlow from "@/components/flow/app-groupe.flow";
import EdgeWithExtremeLabels from "@/components/flow/custom-edge.flow";
import IngestNodeFlow from "@/components/flow/ingest-node.flow";
import MachineNodeFlow from "@/components/flow/machine.flow";
import ProxyNodeFlow from "@/components/flow/proxy-node.flow";
import ServiceFlow from "@/components/flow/service.flow";
import { PROJECT_FRAGMENT } from "@/gql/project.graphql";
import { useProject } from "@/hooks/useProject";
import { ProjectQuery } from "@/router/layout/project.layout";
import type { RouteParams } from "@/router/router.param";
import type { Service } from "@/schemas/service.schema";
import { useFragment, useQuery } from "@apollo/client";
import {
  ReactFlow,
  BackgroundVariant,
  Background,
  type Node,
  Controls,
  useReactFlow,
  useNodesState,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import throttle from "lodash/throttle";
import { useEffect, useLayoutEffect } from "react";
import { useParams } from "react-router-dom";

const APP_GROUP_X_PADDING = 20;
const APP_GROUP_Y_PADDING = 20;
const APP_HEADER_HEIGHT = 35;

const SERVICE_WIDTH = 410;
const SERVICE_HEIGHT = 240;

const edgeTypes = {
  edgeWithExtremeLabels: EdgeWithExtremeLabels,
};

const nodeTypes = {
  ingest: IngestNodeFlow,
  machine: MachineNodeFlow,
  proxy: ProxyNodeFlow,
  application_group: AppGroupFlow,
  service: ServiceFlow,
};

const getNodes = (services: Service[]) => {
  const nodes: Node[] = [];

  nodes.push({
    id: "app",
    position: { x: 0, y: 0 },
    style: {
      width:
        2 * APP_GROUP_X_PADDING +
        services.length * SERVICE_WIDTH +
        (services.length - 1) * APP_GROUP_X_PADDING,
      height:
        2 * APP_GROUP_Y_PADDING + SERVICE_HEIGHT + APP_HEADER_HEIGHT,
    },
    data: { label: "Portfolio" },
    type: "application_group",
  });

  let index = 0;
  for (const service of services) {
    nodes.push({
      id: service.id,
      position: {
        x:
          APP_GROUP_X_PADDING +
          index * SERVICE_WIDTH +
          (index != 0 ? APP_GROUP_X_PADDING : 0),
        y: APP_GROUP_Y_PADDING + APP_HEADER_HEIGHT,
      },
      style: {
        width: SERVICE_WIDTH,
        height: SERVICE_HEIGHT,
      },
      data: { label: service.name },
      parentId: "app",
      extent: "parent" as const,
      type: "service",
    });
    index++;
  }

  return nodes;
};

export default function ProjectFlow() {
  const { projectId } = useParams<RouteParams>() as {
    projectId: string;
  };

  const { data } = useQuery(ProjectQuery, {
    variables: { projectId },
  });

  const project = data?.getProjectById;

  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const reactFlow = useReactFlow();

  useEffect(() => {
    if (project) {
      const newNodes = getNodes(project.services);
      console.log(newNodes);
      if (project.services.length == 0) {
        // dont show the app group
        setNodes([]);
      } else {
        setNodes(newNodes);
      }
    }
  }, [project?.id, project?.services.length]);

  useLayoutEffect(() => {
    // throttle the fitView to 100ms
    const resizeListener = throttle(() => {
      reactFlow.fitView({
        duration: 300,
        padding: 0.3,
      });
    }, 100);
    window.addEventListener("resize", resizeListener);
    return () => {
      window.removeEventListener("resize", resizeListener);
    };
  }, []);

  if (!project) {
    return null;
  }

  return (
    <div className="h-full w-full border-t border-gray-200">
      <ReactFlow
        nodes={nodes}
        edges={[]}
        edgeTypes={edgeTypes}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{
          padding: 0.3,
        }}
        proOptions={{
          hideAttribution: true,
        }}>
        <Background
          variant={BackgroundVariant.Dots}
          gap={10}
          color="#dddddd"
          size={2}
        />
        <Controls
          fitViewOptions={{
            duration: 300,
            padding: 0.3,
          }}
        />
      </ReactFlow>
    </div>
  );
}
