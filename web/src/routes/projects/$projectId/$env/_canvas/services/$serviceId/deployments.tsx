import { createFileRoute } from "@tanstack/react-router";

import { DeploymentList } from "~/components/deployments/deployment-list";

export const Route = createFileRoute(
  "/projects/$projectId/$env/_canvas/services/$serviceId/deployments",
)({
  component: DeploymentsTab,
});

function DeploymentsTab() {
  const { serviceId } = Route.useParams();
  return <DeploymentList serviceId={serviceId} />;
}
