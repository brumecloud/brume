import {
  ReactFlow,
  BackgroundVariant,
  Background,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import React from "react";

const initialNodes = [
  {
    id: "ingest",
    position: { x: 45, y: 0 },
    data: { label: "Brume Ingest" },
  },
  {
    id: "machine",
    position: { x: 0, y: 100 },
    data: { label: "Machine" },
    style: {
      width: 240,
      height: 100,
    },
    type: "group",
  },
  {
    id: "app",
    position: { x: 20, y: 20 },
    style: {
      width: 200,
      height: 60,
    },
    data: { label: "Portfolio" },
    type: "group",
    parentId: "machine",
    extent: "parent",
  },
  {
    id: "user-api",
    position: { x: 25, y: 10 },
    data: { label: "User API" },
    parentId: "app",
    extent: "parent",
  },
];
const initialEdges = [
  { id: "e1-2", source: "ingest", target: "user-api" },
];

export default function ProjectFlow() {
  return (
    <div className="h-full w-full rounded-lg border border-gray-200">
      <ReactFlow
        nodes={initialNodes}
        edges={initialEdges}
        proOptions={{
          hideAttribution: true,
        }}
        fitView>
        <Background
          variant={BackgroundVariant.Dots}
          gap={5}
          color="#f0f0f0"
          size={1}
        />
      </ReactFlow>
    </div>
  );
}
