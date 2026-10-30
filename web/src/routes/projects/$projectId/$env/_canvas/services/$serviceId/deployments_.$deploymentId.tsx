import { createFileRoute, notFound } from "@tanstack/react-router";

import { deploymentsCollection } from "~/api/deployments";
import { DeploymentDetails } from "~/components/deployments/deployment-details";
import { parseDeploymentId, parseDeploymentSearch } from "~/helpers/deployments";

export const Route = createFileRoute(
  "/projects/$projectId/$env/_canvas/services/$serviceId/deployments_/$deploymentId",
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
  const { deploymentId } = Route.useParams();
  const { tab } = Route.useSearch();
  return <DeploymentDetails deploymentId={deploymentId} tab={tab} />;
}
