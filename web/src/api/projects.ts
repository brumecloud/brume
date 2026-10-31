import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection, eq } from "@tanstack/react-db";

import { parseCreatedProject } from "~/helpers/new-project";
import { parseEnvironments, parseProjects } from "~/helpers/projects";

import { apiGet, apiPost } from "./client";
import { queryClient } from "./query-client";
import { servicesCollection } from "./services";
import type { CreatedProject, NewProjectSource, ProjectId } from "./types";

export const projectsCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["projects"],
    queryFn: () => apiGet("/api/projects", parseProjects),
    queryClient,
    getKey: (project) => project.id,
  }),
);

export const environmentsCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["environments"],
    queryFn: () => apiGet("/api/environments", parseEnvironments),
    queryClient,
    getKey: (environment) => environment.id,
  }),
);

export const allEnvironments = (q: InitialQueryBuilder) =>
  q.from({ environment: environmentsCollection });

export const allProjects = (q: InitialQueryBuilder) =>
  q.from({ project: projectsCollection }).orderBy(({ project }) => project.name);

export const projectById = (projectId: ProjectId) => (q: InitialQueryBuilder) =>
  q
    .from({ project: projectsCollection })
    .where(({ project }) => eq(project.id, projectId))
    .findOne();

export const projectEnvironments = (projectId: ProjectId) => (q: InitialQueryBuilder) =>
  q
    .from({ environment: environmentsCollection })
    .where(({ environment }) => eq(environment.projectId, projectId))
    .orderBy(({ environment }) => environment.name);

// The server creates the project, its production environment and its first service in one go:
// refetch the three collections rather than guessing their content.
export async function createProject(source: NewProjectSource): Promise<CreatedProject> {
  const created = await apiPost("/api/projects", source, parseCreatedProject);
  await Promise.all([
    projectsCollection.utils.refetch(),
    environmentsCollection.utils.refetch(),
    servicesCollection.utils.refetch(),
  ]);
  return created;
}
