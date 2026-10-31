import { useMatchRoute } from "@tanstack/react-router";

import type { ProjectId } from "~/api/types";

// The pages of an environment the rail links to.
export const PROJECT_PAGES = {
  canvas: "/projects/$projectId/$env",
  logs: "/projects/$projectId/$env/logs",
  variables: "/projects/$projectId/$env/variables",
  activity: "/projects/$projectId/$env/activity",
} as const;

// The page on screen. The canvas owns every other route of the environment, service panels
// included.
export function useProjectPage(
  projectId: ProjectId,
  environmentName: string,
): keyof typeof PROJECT_PAGES {
  const matchRoute = useMatchRoute();
  const params = { projectId, env: environmentName };
  if (matchRoute({ to: PROJECT_PAGES.logs, params })) {
    return "logs";
  }
  if (matchRoute({ to: PROJECT_PAGES.variables, params })) {
    return "variables";
  }
  return matchRoute({ to: PROJECT_PAGES.activity, params }) ? "activity" : "canvas";
}
