import { createFileRoute, notFound, redirect } from "@tanstack/react-router";

import { environmentsCollection } from "~/api/projects";
import { defaultEnvironment, parseProjectId } from "~/helpers/projects";

export const Route = createFileRoute("/projects/$projectId/")({
  params: {
    parse: (raw) => {
      const projectId = parseProjectId(raw.projectId);
      if (!projectId) {
        throw notFound();
      }
      return { projectId };
    },
    stringify: ({ projectId }) => ({ projectId }),
  },
  loader: async ({ params: { projectId } }) => {
    await environmentsCollection.preload();
    const environment = defaultEnvironment(environmentsCollection.toArray, projectId);
    if (!environment) {
      throw notFound();
    }
    throw redirect({
      to: "/projects/$projectId/$env",
      params: { projectId, env: environment.name },
    });
  },
});
