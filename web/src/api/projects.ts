import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection, eq } from "@tanstack/react-db";

import { parseEnvironments, parseProjects } from "~/helpers/projects";

import { apiGet } from "./client";
import { queryClient } from "./query-client";
import type { ProjectId } from "./types";

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
