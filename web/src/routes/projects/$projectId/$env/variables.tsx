import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { variablesCollection } from "~/api/variables";
import { SharedVariablesView } from "~/components/shared-variables/shared-variables-view";

const projectRoute = getRouteApi("/projects/$projectId/$env");

export const Route = createFileRoute("/projects/$projectId/$env/variables")({
  loader: () => variablesCollection.preload(),
  component: VariablesPage,
});

function VariablesPage() {
  const { projectId } = Route.useParams();
  const { environment } = projectRoute.useLoaderData();
  return <SharedVariablesView projectId={projectId} environment={environment} />;
}
