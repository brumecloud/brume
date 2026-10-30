import { createFileRoute, notFound } from "@tanstack/react-router";

import { deploymentsCollection } from "~/api/deployments";
import { DeploymentPanel } from "~/components/deployments/deployment-panel";
import { parseDeploymentId, parseDeploymentSearch } from "~/helpers/deployments";

export const Route = createFileRoute(
  "/projects/$projectId/$env/_canvas/services/$serviceId/deployments/$deploymentId",
)({
  params: {
    parse: (raw) => {
      const deploymentId = parseDeploymentId(raw.deploymentId);
      if (!deploymentId) {
        throw notFound();
      }
      return { deploymentId };
    },
    stringify: ({ deploymentId }) => ({ deploymentId }),
  },
  validateSearch: parseDeploymentSearch,
  loader: async ({ params: { serviceId, deploymentId } }) => {
    await deploymentsCollection.preload();
    // A deployment of another service is not found here.
    if (deploymentsCollection.get(deploymentId)?.serviceId !== serviceId) {
      throw notFound();
    }
  },
  component: DeploymentPage,
});

function DeploymentPage() {
  const { serviceId, deploymentId } = Route.useParams();
  const { tab } = Route.useSearch();
  return <DeploymentPanel serviceId={serviceId} deploymentId={deploymentId} tab={tab} />;
}
