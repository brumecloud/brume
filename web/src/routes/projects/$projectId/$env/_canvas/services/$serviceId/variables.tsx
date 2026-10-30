import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { VariableList } from "~/components/variables/variable-list";

const projectRoute = getRouteApi("/projects/$projectId/$env");

export const Route = createFileRoute(
  "/projects/$projectId/$env/_canvas/services/$serviceId/variables",
)({
  component: VariablesTab,
});

function VariablesTab() {
  const { serviceId } = Route.useParams();
  const { environment } = projectRoute.useLoaderData();
  return <VariableList serviceId={serviceId} environment={environment} />;
}
