import { withStagedSettings } from "~/helpers/changes";
import { parseImageReference } from "~/helpers/new-project";
import { parseEnvironmentName } from "~/helpers/projects";
import { isDatabase, serviceNameError } from "~/helpers/services";
import { parseVariableKind, parseVariableName, resolveVariables } from "~/helpers/variables";

import type {
  ActivityEvent,
  ActivityPage,
  ChangeId,
  CreatedProject,
  Deployment,
  DeploymentId,
  NewChange,
  Revision,
  ServiceSettings,
  StagedChange,
  Variable,
  VariableId,
  VariableImpact,
  VariableKind,
  VariableLevel,
  Environment,
  EnvironmentId,
  NewProjectSource,
  Project,
  ProjectId,
  Service,
  ServiceId,
  ServiceKind,
} from "~/api/types";

import { type StoredEvent, deploymentEvent, seedEvents } from "./events";
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
  changes: [] as StagedChange[],
  // Values of the staged secrets, kept apart like the applied ones.
  stagedSecrets: new Map<string, string>(),
  // Settings changed by a deploy; the others come from settingsFor.
  settings: new Map<string, ServiceSettings>(),
  // The engine's log, as the activity view reads it. Every mocked action appends to it.
  events: seedEvents(),
};

type EventFields = Partial<Pick<ActivityEvent, "environmentId" | "serviceId" | "deploymentId">>;

function record(
  projectId: string,
  type: string,
  summary: string,
  fields: EventFields = {},
  actor = "admin",
): void {
  const event: StoredEvent["event"] = {
    seq: (db.events.at(-1)?.event.seq ?? 0) + 1,
    at: new Date().toISOString(),
    type,
    actor,
    environmentId: null,
    serviceId: null,
    deploymentId: null,
    summary,
    ...fields,
  };
  db.events.push({ projectId, event });
}

// The project and the environment of a service, for the events about it.
function scopeOf(serviceId: string) {
  const service = db.services.find(({ id }) => id === serviceId);
  const environment = db.environments.find(({ id }) => id === service?.environmentId);
  return service && environment ? { service, environment } : undefined;
}

function recordDeployment(deployment: Deployment, author = "admin"): void {
  const scope = scopeOf(deployment.serviceId);
  if (scope) {
    const { type, summary } = deploymentEvent(
      deployment,
      scope.service.name,
      scope.environment.name,
    );
    record(
      scope.environment.projectId,
      type,
      summary,
      {
        environmentId: scope.environment.id,
        serviceId: scope.service.id,
        deploymentId: deployment.id,
      },
      author,
    );
  }
}

const MAX_EVENTS_PAGE = 200;
const DEFAULT_EVENTS_PAGE = 100;

// `GET /api/events`: a project's events, newest first, `limit` at a time, older than `before`.
export function listEvents(params: URLSearchParams): ActivityPage | Refusal {
  const projectId = params.get("projectId") ?? "";
  const before = params.has("before") ? Number(params.get("before")) : Number.POSITIVE_INFINITY;
  const limit = params.has("limit") ? Number(params.get("limit")) : DEFAULT_EVENTS_PAGE;
  if (!db.projects.some(({ id }) => id === projectId)) {
    return refuse(404, "Project not found.");
  }
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_EVENTS_PAGE || Number.isNaN(before)) {
    return refuse(400, "limit is 1 to 200, before a sequence number.");
  }
  const older = db.events
    .filter((stored) => stored.projectId === projectId && stored.event.seq < before)
    .map((stored) => stored.event)
    .toReversed();
  const events = older.slice(0, limit);
  return {
    events,
    nextBefore: older.length > limit ? (events.at(-1)?.seq ?? null) : null,
  };
}

// A reveal is audited: it leaves an event, never the value.
export function revealSecret(variableId: string): string | undefined {
  const value = db.secretValues.get(variableId);
  const variable = db.variables.find(({ id }) => id === variableId);
  const projectId = variable && projectOfOwner(variable.level, variable.ownerId);
  if (value !== undefined && variable && projectId) {
    record(projectId, "SecretRevealed", `Revealed ${variable.name}`);
  }
  return value;
}

function projectOfOwner(level: VariableLevel, ownerId: string): string | undefined {
  switch (level) {
    case "project":
      return ownerId;
    case "environment":
      return db.environments.find(({ id }) => id === ownerId)?.projectId;
    default:
      return scopeOf(ownerId)?.environment.projectId;
  }
}

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
    name: uniqueProjectName(serviceName?.toLowerCase() ?? "new-project"),
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
  record(project.id, "ProjectCreated", `Created project ${project.name}`);
  record(project.id, "EnvironmentCreated", `Created environment ${environment.name}`, {
    environmentId: environment.id,
  });
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
    const service: Service = {
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
    };
    db.services.push(service);
    record(project.id, "ServiceCreated", `Created service ${service.name} in production`, {
      environmentId: environment.id,
      serviceId,
    });
    const first = db.deployments.at(-1);
    if (first) {
      recordDeployment(first);
    }
    // The repository picked, not the one guessed from the service name.
    if (source.kind === "github" || source.kind === "static") {
      db.settings.set(serviceId, {
        ...settingsFor(service),
        source: { kind: "github", repository: source.repository, branch: "main", autoDeploy: true },
      });
    }
  }
  return { projectId: project.id, environmentName: environment.name };
}

// What the server answers instead of the entity: the handler turns it into an error response.
export interface Refusal {
  readonly status: 400 | 404 | 409;
  readonly message: string;
}

const refuse = (status: Refusal["status"], message: string): Refusal => ({ status, message });

const bodyOf = (raw: unknown) =>
  (typeof raw === "object" && raw !== null ? raw : {}) as Record<string, unknown>;

// A new environment is empty.
export function createEnvironment(projectId: string, raw: unknown): Environment | Refusal {
  const project = db.projects.find(({ id }) => id === projectId);
  if (!project) {
    return refuse(404, "Project not found.");
  }
  const { name } = bodyOf(raw);
  if (typeof name !== "string" || !parseEnvironmentName(name)) {
    return refuse(400, "Invalid environment name.");
  }
  if (db.environments.some((other) => other.projectId === project.id && other.name === name)) {
    return refuse(409, `An environment named ${name} already exists.`);
  }
  const environment: Environment = {
    id: crypto.randomUUID() as EnvironmentId,
    projectId: project.id,
    name,
  };
  db.environments.push(environment);
  record(project.id, "EnvironmentCreated", `Created environment ${environment.name}`, {
    environmentId: environment.id,
  });
  return environment;
}

// How long the mocked agent takes to start a container.
const START_MS = 1_500;
// The containers the mocked agent is starting, by deployment, with the timer that starts them.
const starting = new Map<DeploymentId, ReturnType<typeof setTimeout>>();

// The server resolves the image, then deploys it: `deploying` first, `active` once it runs.
export function createService(environmentId: string, raw: unknown): Service | Refusal {
  const environment = db.environments.find(({ id }) => id === environmentId);
  if (!environment) {
    return refuse(404, "Environment not found.");
  }
  const body = bodyOf(raw);
  const image = typeof body.image === "string" ? parseImageReference(body.image) : undefined;
  if (!image) {
    return refuse(400, "Invalid image reference.");
  }
  const typed = typeof body.name === "string" ? body.name.trim() : "";
  const nameError = serviceNameError(typed);
  if (nameError) {
    return refuse(400, nameError);
  }
  const taken = new Set(
    db.services
      .filter((service) => service.environmentId === environment.id)
      .map((service) => service.slug),
  );
  if (typed && taken.has(slugOf(typed))) {
    return refuse(409, `A service named ${typed} already exists in this environment.`);
  }
  const name = typed || uniqueName(firstServiceName({ kind: "image", image }) ?? "service", taken);
  const service: Service = {
    id: crypto.randomUUID() as ServiceId,
    environmentId: environment.id,
    name,
    slug: slugOf(name),
    kind: "image",
    status: "deploying",
    domains: [],
    volume: null,
    dependencies: [],
  };
  const deployment: Deployment = {
    id: crypto.randomUUID() as DeploymentId,
    serviceId: service.id,
    status: "deploying",
    createdAt: new Date().toISOString(),
    author: "admin",
    commit: null,
    image,
    rollbackable: true,
  };
  db.services.push(service);
  db.deployments.push(deployment);
  record(
    environment.projectId,
    "ServiceCreated",
    `Created service ${name} in ${environment.name}`,
    {
      environmentId: environment.id,
      serviceId: service.id,
    },
  );
  recordDeployment(deployment);
  db.settings.set(service.id, { ...settingsFor(service), source: { kind: "image", image } });
  starting.set(
    deployment.id,
    setTimeout(() => settle(deployment.id), START_MS),
  );
  return service;
}

// Starts at once the containers the mocked agent is starting: a test does not wait for START_MS.
export function startContainers(): void {
  for (const [deploymentId, timer] of starting) {
    clearTimeout(timer);
    settle(deploymentId);
  }
}

function settle(deploymentId: DeploymentId): void {
  starting.delete(deploymentId);
  const index = db.deployments.findIndex(({ id }) => id === deploymentId);
  const deployment = db.deployments[index];
  if (!deployment || deployment.status !== "deploying") {
    return;
  }
  db.deployments[index] = { ...deployment, status: "active" };
  recordDeployment({ ...deployment, status: "active" });
  const serviceIndex = db.services.findIndex(({ id }) => id === deployment.serviceId);
  const service = db.services[serviceIndex];
  if (service) {
    db.services[serviceIndex] = { ...service, status: "online" };
  }
}

// `Cache (EU)` → `cache-eu`: a DNS label, like the server's `Slug`.
function slugOf(name: string): string {
  const slug = name
    .toLowerCase()
    .replaceAll(/[^a-z0-9]+/g, "-")
    .replaceAll(/^-+|-+$/g, "")
    .slice(0, 63)
    .replace(/-+$/, "");
  return slug || "service";
}

function uniqueName(base: string, taken: ReadonlySet<string>): string {
  let name = base;
  for (let suffix = 2; taken.has(slugOf(name)); suffix += 1) {
    name = `${base}-${suffix}`;
  }
  return name;
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
  recordDeployment(deployment, author);
  const index = db.services.findIndex((service) => service.id === from.serviceId);
  const service = db.services[index];
  if (service) {
    db.services[index] = { ...service, status: "online" };
  }
  return deployment;
}

export function deleteService(serviceId: string): void {
  const scope = scopeOf(serviceId);
  if (scope) {
    record(
      scope.environment.projectId,
      "ServiceDeleted",
      `Deleted service ${scope.service.name} from ${scope.environment.name}`,
      { environmentId: scope.environment.id, serviceId: scope.service.id },
    );
  }
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

const TEMPLATE_VERSIONS = { postgres: "17.6", mongodb: "8.0.15", redis: "8.2.2" } as const;

function settingsFor(service: Service): ServiceSettings {
  const environment = db.environments.find(({ id }) => id === service.environmentId);
  const limits = { cpuLimit: 1, memoryLimitMb: 512 };
  switch (service.kind) {
    case "github":
    case "static": {
      const app = seed.apps[service.name];
      const runtime = service.kind === "github" ? seed.runtimes[app?.runtime ?? "node"] : null;
      return {
        source: {
          kind: "github",
          repository: app?.repository ?? `${service.slug}/${service.slug}`,
          branch: environment?.name === "production" ? "main" : (environment?.name ?? "main"),
          autoDeploy: true,
        },
        startCommand: runtime?.startCommand ?? null,
        port: runtime?.port ?? null,
        healthcheckPath: runtime ? "/health" : null,
        ...limits,
        egress: true,
        domain: null,
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
        domain: null,
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
        domain: null,
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
    startCommand: currentSettings(service).startCommand,
    variables: resolveVariables(db.variables, service, environment).map(({ variable, origin }) => ({
      name: variable.name,
      kind: variable.kind,
      value: variable.value,
      origin: variable.reference
        ? (sharedSecretOrigin(variable.name, environment) ?? origin)
        : origin,
    })),
  };
}

export function currentSettings(service: Service): ServiceSettings {
  return db.settings.get(service.id) ?? settingsFor(service);
}

// What the API sends: never the value of a staged secret.
export function publicChange(change: StagedChange): StagedChange {
  return change.type === "variable" && change.kind === "secret"
    ? { ...change, value: null }
    : change;
}

// One draft per setting and one per variable name: a new edit replaces the previous one.
export function stageChange(raw: unknown): string | StagedChange {
  const body = (typeof raw === "object" && raw !== null ? raw : {}) as Record<string, unknown>;
  const service = db.services.find(({ id }) => id === body.serviceId);
  if (!service) {
    return "Service not found.";
  }
  const input = body as unknown as NewChange;
  const sameTarget = (change: StagedChange) =>
    change.serviceId === service.id &&
    (input.type === "setting"
      ? change.type === "setting" && change.field === input.field
      : change.type !== "setting" && change.name === input.name);
  db.changes = db.changes.filter((change) => !sameTarget(change));
  const change: StagedChange = {
    ...(input.type === "variable"
      ? { ...input, reference: input.kind === "secret" && input.value === null }
      : input),
    id: crypto.randomUUID() as ChangeId,
    environmentId: service.environmentId,
  };
  if (change.type === "variable" && change.kind === "secret" && change.value !== null) {
    db.stagedSecrets.set(change.id, change.value);
  }
  db.changes.push(change);
  return publicChange(change);
}

export function discardChanges(environmentId: string): void {
  db.changes = db.changes.filter((change) => change.environmentId !== environmentId);
}

// Applies every staged change of the environment, then deploys each service it touched.
export function deployChanges(environmentId: string): number {
  const changes = db.changes.filter((change) => change.environmentId === environmentId);
  const touched = new Set(changes.map((change) => change.serviceId));
  for (const serviceId of touched) {
    const index = db.services.findIndex(({ id }) => id === serviceId);
    const service = db.services[index];
    if (!service) {
      continue;
    }
    const own = changes.filter((change) => change.serviceId === serviceId);
    const scope = scopeOf(serviceId);
    if (scope) {
      record(
        scope.environment.projectId,
        "ServiceSpecChanged",
        `Applied ${own.length} ${own.length === 1 ? "change" : "changes"} to ${service.name} in ${scope.environment.name}`,
        { environmentId: scope.environment.id, serviceId },
      );
    }
    const next = withStagedSettings(currentSettings(service), service.domains, own);
    db.settings.set(serviceId, next.settings);
    db.services[index] = { ...service, domains: next.domains };
    for (const change of own) {
      applyVariableChange(change);
    }
    const active = db.deployments.find(
      (deployment) => deployment.serviceId === serviceId && deployment.status === "active",
    );
    if (active) {
      deploy(active, "admin");
    }
  }
  discardChanges(environmentId);
  return touched.size;
}

function applyVariableChange(change: StagedChange): void {
  if (change.type === "setting") {
    return;
  }
  const existing = db.variables.find(
    (variable) => variable.ownerId === change.serviceId && variable.name === change.name,
  );
  db.variables = db.variables.filter((variable) => variable !== existing);
  if (change.type === "variable-removal") {
    return;
  }
  const { reference } = change;
  const variable: Variable = {
    id: existing?.id ?? (crypto.randomUUID() as VariableId),
    level: "service",
    ownerId: change.serviceId,
    name: change.name,
    kind: change.kind,
    value: change.kind === "secret" ? null : change.value,
    reference,
  };
  if (change.kind === "secret" && !reference) {
    db.secretValues.set(variable.id, db.stagedSecrets.get(change.id) ?? "");
    db.stagedSecrets.delete(change.id);
  }
  db.variables.push(variable);
}

// Where a referenced secret resolves: the environment, then the project.
function sharedSecretOrigin(name: string, environment: Environment): VariableLevel | undefined {
  const holds = (level: VariableLevel, ownerId: string) =>
    db.variables.some(
      (variable) =>
        variable.level === level &&
        variable.ownerId === ownerId &&
        variable.name === name &&
        variable.kind === "secret",
    );
  if (holds("environment", environment.id)) {
    return "environment";
  }
  return holds("project", environment.projectId) ? "project" : undefined;
}

type SharedLevel = Extract<VariableLevel, "project" | "environment">;

const MAX_VALUE_BYTES = 32 * 1024;

// The services a shared value reaches. A plain or build value reaches every service that does not
// override it, databases aside; a secret only the services that reference it.
function reachedServices(
  level: SharedLevel,
  ownerId: string,
  name: string,
  kind: VariableKind,
): Service[] {
  const environments = db.environments.filter((environment) =>
    level === "project" ? environment.projectId === ownerId : environment.id === ownerId,
  );
  const named = db.variables.filter((variable) => variable.name === name);
  const overridden = (at: VariableLevel, holder: string) =>
    named.some((variable) => variable.level === at && variable.ownerId === holder);
  return environments.flatMap((environment) =>
    db.services.filter((service) => {
      if (service.environmentId !== environment.id) {
        return false;
      }
      if (level === "project" && overridden("environment", environment.id)) {
        return false;
      }
      if (kind === "secret") {
        return named.some(
          (variable) =>
            variable.level === "service" && variable.ownerId === service.id && variable.reference,
        );
      }
      const reaches = !isDatabase(service.kind) && (service.kind !== "static" || kind === "build");
      return reaches && !overridden("service", service.id);
    }),
  );
}

function sharedOwner(level: unknown, ownerId: unknown): { level: SharedLevel; projectId: string } {
  if (level === "project" && db.projects.some(({ id }) => id === ownerId)) {
    return { level, projectId: String(ownerId) };
  }
  const environment = db.environments.find(({ id }) => id === ownerId);
  return level === "environment" && environment
    ? { level, projectId: environment.projectId }
    : { level: "project", projectId: "" };
}

// The variables of every level of a project.
function projectVariables(projectId: string): Variable[] {
  const environments = new Set<string>(
    db.environments
      .filter((environment) => environment.projectId === projectId)
      .map(({ id }) => id),
  );
  const services = new Set<string>(
    db.services.filter((service) => environments.has(service.environmentId)).map(({ id }) => id),
  );
  return db.variables.filter(
    (variable) =>
      variable.ownerId === projectId ||
      environments.has(variable.ownerId) ||
      services.has(variable.ownerId),
  );
}

export function variableImpact(params: URLSearchParams): VariableImpact | Refusal {
  const level = params.get("level");
  const ownerId = params.get("ownerId") ?? "";
  const name = params.get("name") ?? "";
  const owner = sharedOwner(level, ownerId);
  if (!owner.projectId || !parseVariableName(name)) {
    return refuse(400, "Invalid level, owner or name.");
  }
  const existing = db.variables.find(
    (variable) =>
      variable.level === owner.level && variable.ownerId === ownerId && variable.name === name,
  );
  return {
    services: reachedServices(owner.level, ownerId, name, existing?.kind ?? "plain").map(
      (service) => ({ serviceId: service.id, environmentId: service.environmentId }),
    ),
  };
}

const redeployAll = (services: readonly Service[]): Deployment[] =>
  services.flatMap((service) => {
    const active = db.deployments.find(
      (deployment) => deployment.serviceId === service.id && deployment.status === "active",
    );
    return active ? [deploy(active, "admin")] : [];
  });

export function setSharedVariable(
  raw: unknown,
): { variable: Variable; deployments: Deployment[]; referencedBy: ServiceId[] } | Refusal {
  const body = bodyOf(raw);
  const owner = sharedOwner(body.level, body.ownerId);
  const name = typeof body.name === "string" ? parseVariableName(body.name) : undefined;
  const kind = parseVariableKind(body.kind);
  const value = body.value;
  if (!owner.projectId || !name || !kind || typeof value !== "string") {
    return refuse(400, "Invalid shared variable.");
  }
  if (new TextEncoder().encode(value).length > MAX_VALUE_BYTES) {
    return refuse(400, "A value holds 32 KiB at most.");
  }
  const ownerId = String(body.ownerId);
  const existing = db.variables.find(
    (variable) =>
      variable.level === owner.level && variable.ownerId === ownerId && variable.name === name,
  );
  const clash = projectVariables(owner.projectId).find(
    (variable) =>
      variable !== existing &&
      variable.name === name &&
      (variable.kind === "secret") !== (kind === "secret"),
  );
  if (clash) {
    return refuse(
      409,
      `${name} is already a ${clash.kind === "secret" ? "secret" : "variable"} in this project.`,
    );
  }
  const variable: Variable = {
    id: existing?.id ?? (crypto.randomUUID() as VariableId),
    level: owner.level,
    ownerId,
    name,
    kind,
    value: kind === "secret" ? null : value,
    reference: false,
  };
  db.variables = [...db.variables.filter((other) => other !== existing), variable];
  record(owner.projectId, "SharedVariableSet", `Set ${name} on ${sharedOwnerLabel(variable)}`, {
    environmentId: variable.level === "environment" ? (ownerId as EnvironmentId) : null,
  });
  const reached = reachedServices(owner.level, ownerId, name, kind);
  if (kind === "secret") {
    db.secretValues.set(variable.id, value);
    return { variable, deployments: [], referencedBy: reached.map((service) => service.id) };
  }
  db.secretValues.delete(variable.id);
  return { variable, deployments: redeployAll(reached), referencedBy: [] };
}

export function removeSharedVariable(variableId: string): { deployments: Deployment[] } | Refusal {
  const variable = db.variables.find(({ id }) => id === variableId);
  if (!variable) {
    return refuse(404, "Variable not found.");
  }
  if (variable.level === "service") {
    return refuse(400, "A service variable changes through a staged change.");
  }
  const reached = reachedServices(variable.level, variable.ownerId, variable.name, variable.kind);
  if (variable.kind === "secret") {
    const users = reached.filter((service) =>
      db.deployments.some(
        (deployment) => deployment.serviceId === service.id && deployment.status === "active",
      ),
    );
    if (users.length > 0) {
      const names = users.map((service) => {
        const environment = db.environments.find(({ id }) => id === service.environmentId);
        return `${service.name} (${environment?.name ?? "?"})`;
      });
      return refuse(409, `${variable.name} is still referenced by ${names.join(", ")}.`);
    }
  }
  db.variables = db.variables.filter((other) => other !== variable);
  const projectId = projectOfOwner(variable.level, variable.ownerId);
  if (projectId) {
    record(
      projectId,
      "SharedVariableRemoved",
      `Removed ${variable.name} from ${sharedOwnerLabel(variable)}`,
      {
        environmentId:
          variable.level === "environment" ? (variable.ownerId as EnvironmentId) : null,
      },
    );
  }
  db.secretValues.delete(variable.id);
  return { deployments: variable.kind === "secret" ? [] : redeployAll(reached) };
}

function sharedOwnerLabel(variable: Variable): string {
  return variable.level === "project"
    ? "the project"
    : (db.environments.find(({ id }) => id === variable.ownerId)?.name ?? "an environment");
}
