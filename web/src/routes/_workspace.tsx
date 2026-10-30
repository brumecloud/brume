import { Outlet, createFileRoute } from "@tanstack/react-router";

import { WorkspaceShell } from "~/components/layout/workspace-shell";

export const Route = createFileRoute("/_workspace")({
  component: WorkspaceLayout,
});

function WorkspaceLayout() {
  return (
    <WorkspaceShell>
      <Outlet />
    </WorkspaceShell>
  );
}
