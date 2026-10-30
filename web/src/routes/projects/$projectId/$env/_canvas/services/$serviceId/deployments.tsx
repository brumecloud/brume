import { Outlet, createFileRoute } from "@tanstack/react-router";

import { DeploymentList } from "~/components/deployments/deployment-list";

export const Route = createFileRoute(
  "/projects/$projectId/$env/_canvas/services/$serviceId/deployments",
)({
  component: DeploymentsTab,
});

// A deployment opens in a panel stacked over this list.
function DeploymentsTab() {
  const { serviceId } = Route.useParams();
  return (
    <>
      <DeploymentList serviceId={serviceId} />
      <Outlet />
    </>
  );
}
