import type {
  CreatedProject,
  Environment,
  EnvironmentId,
  NewProjectSource,
  Project,
  ProjectId,
  Service,
  ServiceId,
  ServiceKind,
} from "~/api/types";

import * as seed from "./seed";

// In-memory state of the mocked server. A page reload resets it to the seed.
export const db = {
  projects: [...seed.projects],
  environments: [...seed.environments],
  services: [...seed.services],
};

const SERVICE_NAMES: Record<Exclude<ServiceKind, "github" | "static" | "image">, string> = {
  postgres: "PostgreSQL",
  mongodb: "MongoDB",
  redis: "Redis",
};

// Mirrors what the server will do: a project, its production environment and, unless empty,
// a first service built from the source.
export function createProject(source: NewProjectSource): CreatedProject {
  const serviceName = firstServiceName(source);
  const project: Project = {
    id: crypto.randomUUID() as ProjectId,
    name: uniqueProjectName(serviceName?.toLowerCase() ?? "nouveau-projet"),
    description: "",
    updatedAt: new Date().toISOString(),
  };
  const environment: Environment = {
    id: crypto.randomUUID() as EnvironmentId,
    projectId: project.id,
    name: "production",
  };
  db.projects.push(project);
  db.environments.push(environment);
  if (serviceName && source.kind !== "empty") {
    db.services.push({
      id: crypto.randomUUID() as ServiceId,
      environmentId: environment.id,
      name: serviceName,
      slug: serviceName.toLowerCase(),
      kind: source.kind,
      status: source.kind === "github" || source.kind === "static" ? "deploying" : "online",
    } satisfies Service);
  }
  return { projectId: project.id, environmentName: environment.name };
}

function firstServiceName(source: NewProjectSource): string | undefined {
  switch (source.kind) {
    case "empty":
      return undefined;
    case "github":
    case "static":
      return source.repository.split("/").at(-1);
    case "image": {
      // `ghcr.io/team/app:1.2@sha256:…` → `app`
      const [path = source.image] = source.image.split("@");
      const name = path.split("/").at(-1) ?? path;
      return name.split(":")[0] ?? name;
    }
    default:
      return SERVICE_NAMES[source.kind];
  }
}

function uniqueProjectName(base: string): string {
  const taken = new Set(db.projects.map((project) => project.name));
  let name = base;
  for (let suffix = 2; taken.has(name); suffix += 1) {
    name = `${base}-${suffix}`;
  }
  return name;
}

// The server validates the body: so does the mock, loosely.
export function readNewProjectSource(raw: unknown): NewProjectSource | undefined {
  if (typeof raw !== "object" || raw === null || !("kind" in raw)) {
    return undefined;
  }
  const source = raw as Record<string, unknown>;
  switch (source.kind) {
    case "empty":
    case "postgres":
    case "mongodb":
    case "redis":
      return { kind: source.kind };
    case "github":
    case "static":
      return typeof source.repository === "string"
        ? { kind: source.kind, repository: source.repository }
        : undefined;
    case "image":
      return typeof source.image === "string" ? { kind: "image", image: source.image } : undefined;
    default:
      return undefined;
  }
}
