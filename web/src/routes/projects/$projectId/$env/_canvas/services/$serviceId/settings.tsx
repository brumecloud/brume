import { createFileRoute } from "@tanstack/react-router";

import { ingressQuery } from "~/api/ingress";
import { queryClient } from "~/api/query-client";
import { serviceSettingsQuery } from "~/api/services";
import { ServiceSettingsView } from "~/components/service-settings/service-settings-view";

export const Route = createFileRoute(
  "/projects/$projectId/$env/_canvas/services/$serviceId/settings",
)({
  loader: ({ params }) =>
    Promise.all([
      queryClient.ensureQueryData(serviceSettingsQuery(params.serviceId)),
      queryClient.ensureQueryData(ingressQuery),
    ]),
  component: SettingsTab,
});

function SettingsTab() {
  const { projectId, env, serviceId } = Route.useParams();
  return <ServiceSettingsView projectId={projectId} environmentName={env} serviceId={serviceId} />;
}
