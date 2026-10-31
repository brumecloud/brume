import { Outlet, createFileRoute, notFound } from "@tanstack/react-router";

import { deploymentsCollection } from "~/api/deployments";
import { environmentsCollection } from "~/api/projects";
import { servicesCollection } from "~/api/services";
import { variablesCollection } from "~/api/variables";
import { ServicePanel } from "~/components/service-panel/service-panel";
import { findEnvironment } from "~/helpers/projects";
import { parseServiceId } from "~/helpers/services";

export const Route = createFileRoute("/projects/$projectId/$env/_canvas/services/$serviceId")({
  params: {
    parse: (raw) => {
      const serviceId = parseServiceId(raw.serviceId);
      if (!serviceId) {
        throw notFound();
      }
      return { serviceId };
    },
    stringify: ({ serviceId }) => ({ serviceId }),
  },
  loader: async ({ params: { projectId, env, serviceId } }) => {
    // Loaders run in parallel: each one waits for the collections it reads.
    await Promise.all([
      environmentsCollection.preload(),
      servicesCollection.preload(),
      deploymentsCollection.preload(),
      variablesCollection.preload(),
    ]);
    const environment = findEnvironment(environmentsCollection.toArray, projectId, env);
    // A service id from another environment is not found here.
    if (!environment || servicesCollection.get(serviceId)?.environmentId !== environment.id) {
      throw notFound();
    }
  },
  component: ServicePanelLayout,
});

function ServicePanelLayout() {
  const { projectId, env, serviceId } = Route.useParams();
  return (
    <ServicePanel projectId={projectId} environmentName={env} serviceId={serviceId}>
      <Outlet />
    </ServicePanel>
  );
}
