import { createFileRoute } from "@tanstack/react-router";

import { activityQuery } from "~/api/activity";
import { queryClient } from "~/api/query-client";
import { ActivityList } from "~/components/activity/activity-list";

// The project's log, whatever the environment in the URL: the shell keeps its switcher and rail.
// Fetched on every visit: the log only grows.
export const Route = createFileRoute("/projects/$projectId/$env/activity")({
  loader: ({ params: { projectId } }) => queryClient.fetchInfiniteQuery(activityQuery(projectId)),
  component: ActivityPage,
});

function ActivityPage() {
  const { projectId } = Route.useParams();
  return <ActivityList projectId={projectId} />;
}
