import { resolveVariables } from "~/helpers/variables";

import type {
  CreatedProject,
  Deployment,
  DeploymentId,
  Revision,
  ServiceSettings,
  Variable,
  VariableId,
  VariableKind,
  Environment,
  EnvironmentId,
  NewProjectSource,
  Project,
  ProjectId,
  Service,
  ServiceId,
  ServiceKind,
} from "~/api/types";

import * as history from "./history";
import * as seed from "./seed";

// In-memory state of the mocked server. A page reload resets it to the seed.
export const db = {
  projects: [...seed.projects],
  environments: [...seed.environments],
  services: [...seed.services],
  deployments: [...history.deployments],
  variables: [...history.variables],
  secretValues: new Map(history.secretValues),
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
    const serviceId = crypto.randomUUID() as ServiceId;
    db.deployments.push({
      id: crypto.randomUUID() as DeploymentId,
      serviceId,
      status: source.kind === "github" || source.kind === "static" ? "deploying" : "active",
      createdAt: new Date().toISOString(),
      author: "admin",
      commit: null,
      image: source.kind === "image" ? source.image : null,
      rollbackable: true,
    });
    db.services.push({
      id: serviceId,
      environmentId: environment.id,
      name: serviceName,
      slug: serviceName.toLowerCase(),
      kind: source.kind,
      status: source.kind === "github" || source.kind === "static" ? "deploying" : "online",
      domains: [],
      volume:
        source.kind in SERVICE_NAMES
          ? { name: `${serviceName.toLowerCase()}-data`, mountPath: "/data" }
          : null,
      dependencies: [],
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

// Puts a deployment in service: the one it replaces is kept in the history.
export function deploy(from: Deployment, author: string): Deployment {
  for (const [index, deployment] of db.deployments.entries()) {
    if (deployment.serviceId === from.serviceId && deployment.status === "active") {
      db.deployments[index] = { ...deployment, status: "superseded" };
    }
  }
  const deployment: Deployment = {
    ...from,
    id: crypto.randomUUID() as DeploymentId,
    status: "active",
    createdAt: new Date().toISOString(),
    author,
  };
  db.deployments.push(deployment);
  const index = db.services.findIndex((service) => service.id === from.serviceId);
  const service = db.services[index];
  if (service) {
    db.services[index] = { ...service, status: "online" };
  }
  return deployment;
}

export function deleteService(serviceId: string): void {
  db.services = db.services.filter((service) => service.id !== serviceId);
  for (const [index, service] of db.services.entries()) {
    if (service.dependencies.includes(serviceId as ServiceId)) {
      db.services[index] = {
        ...service,
        dependencies: service.dependencies.filter((dependency) => dependency !== serviceId),
      };
    }
  }
  db.deployments = db.deployments.filter((deployment) => deployment.serviceId !== serviceId);
  db.variables = db.variables.filter((variable) => variable.ownerId !== serviceId);
}

const KINDS: ReadonlySet<string> = new Set<VariableKind>(["plain", "build", "secret"]);
const NAME = /^[A-Za-z_][A-Za-z0-9_]{0,127}$/;

// Returns an error message the panel shows as is, or the created variable.
export function createVariable(raw: unknown): string | Variable {
  const body = (typeof raw === "object" && raw !== null ? raw : {}) as Record<string, unknown>;
  const { serviceId, name, kind, value } = body;
  if (
    typeof serviceId !== "string" ||
    typeof name !== "string" ||
    typeof value !== "string" ||
    typeof kind !== "string" ||
    !KINDS.has(kind) ||
    !NAME.test(name)
  ) {
    return "Variable invalide.";
  }
  if (db.variables.some((variable) => variable.ownerId === serviceId && variable.name === name)) {
    return `${name} existe déjà sur ce service.`;
  }
  const variable: Variable = {
    id: crypto.randomUUID() as VariableId,
    level: "service",
    ownerId: serviceId,
    name,
    kind: kind as VariableKind,
    value: kind === "secret" ? null : value,
  };
  if (kind === "secret") {
    db.secretValues.set(variable.id, value);
  }
  db.variables.push(variable);
  return variable;
}

const TEMPLATE_VERSIONS = { postgres: "17.6", mongodb: "8.0.15", redis: "8.2.2" } as const;

export function settingsFor(service: Service): ServiceSettings {
  const environment = db.environments.find(({ id }) => id === service.environmentId);
  const limits = { cpuLimit: 1, memoryLimitMb: 512 };
  switch (service.kind) {
    case "github":
    case "static": {
      const project = service.name.split("-")[0] ?? service.name;
      return {
        source: {
          kind: "github",
          repository: project === "brume" ? "brumecloud/brume" : `planchon/${project}`,
          branch: environment?.name === "production" ? "main" : (environment?.name ?? "main"),
          autoDeploy: true,
        },
        startCommand: service.kind === "github" ? "pnpm start" : null,
        port: service.kind === "github" ? 3000 : null,
        healthcheckPath: service.kind === "github" ? "/health" : null,
        ...limits,
        egress: true,
      };
    }
    case "image": {
      const image = db.deployments.find(
        (deployment) => deployment.serviceId === service.id && deployment.status === "active",
      )?.image;
      return {
        source: { kind: "image", image: image ?? service.name },
        startCommand: null,
        port: null,
        healthcheckPath: null,
        ...limits,
        egress: true,
      };
    }
    default:
      return {
        source: {
          kind: "template",
          template: service.kind,
          version: TEMPLATE_VERSIONS[service.kind],
        },
        startCommand: null,
        port: null,
        healthcheckPath: null,
        cpuLimit: 1,
        memoryLimitMb: 1024,
        egress: false,
      };
  }
}

// The spec a deployment put in service: its variables as resolved at that time.
export function revisionFor(deployment: Deployment): Revision | undefined {
  const service = db.services.find(({ id }) => id === deployment.serviceId);
  const environment = db.environments.find(({ id }) => id === service?.environmentId);
  if (!service || !environment) {
    return undefined;
  }
  return {
    digest: `sha256:${(deployment.commit?.sha ?? deployment.id.replaceAll("-", "")).repeat(8).slice(0, 64)}`,
    builder: deployment.commit ? "Railpack 0.40.1" : null,
    startCommand: settingsFor(service).startCommand,
    variables: resolveVariables(db.variables, service, environment).map(({ variable, origin }) => ({
      name: variable.name,
      kind: variable.kind,
      value: variable.value,
      origin,
    })),
  };
}
