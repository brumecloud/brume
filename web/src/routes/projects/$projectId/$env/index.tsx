import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { ServiceCanvas } from "~/components/canvas/service-canvas";

const projectRoute = getRouteApi("/projects/$projectId/$env");

export const Route = createFileRoute("/projects/$projectId/$env/")({
  component: CanvasPage,
});

function CanvasPage() {
  const { environment } = projectRoute.useLoaderData();
  return <ServiceCanvas environmentId={environment.id} />;
}
