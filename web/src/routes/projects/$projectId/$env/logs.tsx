import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { LogExplorer } from "~/components/logs/log-explorer";
import { parseLogsSearch } from "~/helpers/logs";

const projectRoute = getRouteApi("/projects/$projectId/$env");

export const Route = createFileRoute("/projects/$projectId/$env/logs")({
  validateSearch: parseLogsSearch,
  component: LogsPage,
});

function LogsPage() {
  const { environment } = projectRoute.useLoaderData();
  const search = Route.useSearch();
  return <LogExplorer environmentId={environment.id} search={search} />;
}
