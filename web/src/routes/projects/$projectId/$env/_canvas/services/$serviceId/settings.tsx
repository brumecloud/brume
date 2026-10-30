import { createFileRoute } from "@tanstack/react-router";

import { queryClient } from "~/api/query-client";
import { serviceSettingsQuery } from "~/api/services";
import { ServiceSettingsView } from "~/components/service-settings/service-settings-view";

export const Route = createFileRoute(
  "/projects/$projectId/$env/_canvas/services/$serviceId/settings",
)({
  loader: ({ params }) => queryClient.ensureQueryData(serviceSettingsQuery(params.serviceId)),
  component: SettingsTab,
});

function SettingsTab() {
  const { projectId, env, serviceId } = Route.useParams();
  return <ServiceSettingsView projectId={projectId} environmentName={env} serviceId={serviceId} />;
}
