import { Outlet, createFileRoute, notFound } from "@tanstack/react-router";

import { environmentsCollection, projectsCollection } from "~/api/projects";
import { servicesCollection } from "~/api/services";
import { ProjectShell } from "~/components/layout/project-shell";
import { findEnvironment, parseEnvironmentName, parseProjectId } from "~/helpers/projects";

export const Route = createFileRoute("/projects/$projectId/$env")({
  params: {
    parse: (raw) => {
      const projectId = parseProjectId(raw.projectId);
      const env = parseEnvironmentName(raw.env);
      if (!projectId || !env) {
        throw notFound();
      }
      return { projectId, env };
    },
    stringify: ({ projectId, env }) => ({ projectId, env }),
  },
  loader: async ({ params: { projectId, env } }) => {
    await Promise.all([
      projectsCollection.preload(),
      environmentsCollection.preload(),
      servicesCollection.preload(),
    ]);
    const environment = findEnvironment(environmentsCollection.toArray, projectId, env);
    if (!projectsCollection.has(projectId) || !environment) {
      throw notFound();
    }
    return { environment };
  },
  component: ProjectLayout,
});

function ProjectLayout() {
  const { projectId, env } = Route.useParams();
  return (
    <ProjectShell projectId={projectId} environmentName={env}>
      <Outlet />
    </ProjectShell>
  );
}
