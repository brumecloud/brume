import { Outlet, createFileRoute, getRouteApi } from "@tanstack/react-router";

import { ServiceCanvas } from "~/components/canvas/service-canvas";
import { StagedChangesBar } from "~/components/changes/staged-changes-bar";
import { PanelLayer } from "~/components/layout/panel-layer";

const projectRoute = getRouteApi("/projects/$projectId/$env");

export const Route = createFileRoute("/projects/$projectId/$env/_canvas")({
  component: CanvasLayout,
});

// The service panel opens over the canvas, which stays mounted behind it; a deployment opens
// over the service panel.
function CanvasLayout() {
  const { projectId, env } = Route.useParams();
  const { environment } = projectRoute.useLoaderData();

  return (
    <PanelLayer>
      <ServiceCanvas projectId={projectId} environmentName={env} environmentId={environment.id} />
      <Outlet />
      <StagedChangesBar environmentId={environment.id} />
    </PanelLayer>
  );
}
