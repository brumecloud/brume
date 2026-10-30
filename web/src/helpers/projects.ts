import type { Environment, EnvironmentId, Project, ProjectId } from "~/api/types";

import { decodeArray, decodeRecord, readString } from "./decode";
import { isId } from "./ids";

const DNS_LABEL = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/;
const DEFAULT_ENVIRONMENT = "production";

export function parseProjects(raw: unknown): Project[] {
  return decodeArray(raw, parseProject, "projects");
}

function parseProject(raw: unknown): Project {
  const record = decodeRecord(raw, "project");
  return {
    id: readString(record, "id", "project") as ProjectId,
    name: readString(record, "name", "project"),
    description: readString(record, "description", "project"),
    updatedAt: readString(record, "updatedAt", "project"),
  };
}

export function parseEnvironments(raw: unknown): Environment[] {
  return decodeArray(raw, parseEnvironment, "environments");
}

function parseEnvironment(raw: unknown): Environment {
  const record = decodeRecord(raw, "environment");
  return {
    id: readString(record, "id", "environment") as EnvironmentId,
    projectId: readString(record, "projectId", "environment") as ProjectId,
    name: readString(record, "name", "environment"),
  };
}

// URL params are hostile: a crafted link can be sent to the admin.
export function parseProjectId(raw: string): ProjectId | undefined {
  return isId(raw) ? (raw as ProjectId) : undefined;
}

export function parseEnvironmentName(raw: string): string | undefined {
  return DNS_LABEL.test(raw) ? raw : undefined;
}

export function defaultEnvironment(
  environments: readonly Environment[],
  projectId: ProjectId,
): Environment | undefined {
  const own = environments
    .filter((environment) => environment.projectId === projectId)
    .toSorted((a, b) => a.name.localeCompare(b.name));
  return own.find((environment) => environment.name === DEFAULT_ENVIRONMENT) ?? own[0];
}

export function findEnvironment(
  environments: readonly Environment[],
  projectId: ProjectId,
  name: string,
): Environment | undefined {
  return environments.find(
    (environment) => environment.projectId === projectId && environment.name === name,
  );
}
