import { createFileRoute } from "@tanstack/react-router";
import { WorkflowIcon } from "lucide-react";

import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";

export const Route = createFileRoute("/projects/$projectId/$env/")({
  component: CanvasPage,
});

// Placeholder until the React Flow canvas (batch 2).
function CanvasPage() {
  return (
    <Empty className="h-full">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <WorkflowIcon />
        </EmptyMedia>
        <EmptyTitle>Architecture</EmptyTitle>
        <EmptyDescription>Le canvas des services arrive au lot 2.</EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
