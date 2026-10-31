import type {
  Environment,
  EnvironmentId,
  Project,
  ProjectId,
  Repository,
  Service,
  ServiceId,
  ServiceKind,
  ServiceStatus,
} from "~/api/types";

const project = (id: string, name: string, description: string, updatedAt: string): Project => ({
  id: id as ProjectId,
  name,
  description,
  updatedAt,
});

const environment = (projectId: ProjectId, name: string): Environment => ({
  id: `${projectId}-${name}` as EnvironmentId,
  projectId,
  name,
});

interface ServiceOptions {
  readonly status?: ServiceStatus;
  readonly domains?: readonly string[];
  readonly volume?: string;
  // Names of the services of the same environment this one connects to.
  readonly dependsOn?: readonly string[];
}

const MOUNT_PATHS: Partial<Record<ServiceKind, string>> = {
  postgres: "/var/lib/postgresql/data",
  mongodb: "/data/db",
  redis: "/data",
};

const service = (
  env: Environment,
  name: string,
  kind: ServiceKind,
  { status = "online", domains = [], volume, dependsOn = [] }: ServiceOptions = {},
): Service => ({
  id: `${env.id}-${name}` as ServiceId,
  environmentId: env.id,
  name,
  slug: name.toLowerCase(),
  kind,
  status,
  domains,
  volume: volume ? { name: volume, mountPath: MOUNT_PATHS[kind] ?? "/data" } : null,
  dependencies: dependsOn.map((dependency) => `${env.id}-${dependency}` as ServiceId),
});

export const brume = project(
  "c0a1d9f0-fd9b-4683-b608-2bf60613b499",
  "brume",
  "API, worker, MongoDB et Redis.",
  "2026-10-29T21:12:00Z",
);
const beaubon = project(
  "5b0e7c52-8d0b-4a3e-9a51-3c1f0e8e2d10",
  "beaubon",
  "Beaubon : API Go, app Expo, site web.",
  "2026-10-28T09:40:00Z",
);
const melled = project(
  "9f3c2a71-64b5-4f0e-b2d8-0a7e5c1d4b22",
  "melled",
  "",
  "2026-10-21T16:05:00Z",
);
const eve = project("2d7e9b14-0c3a-4e61-8f5d-6b2a1c9e7f33", "eve", "", "2026-10-12T11:30:00Z");

export const projects: readonly Project[] = [brume, beaubon, melled, eve];

const brumeProduction = environment(brume.id, "production");
const brumeStaging = environment(brume.id, "staging");
const beaubonProduction = environment(beaubon.id, "production");
const melledProduction = environment(melled.id, "production");
const eveDev = environment(eve.id, "dev");

export const environments: readonly Environment[] = [
  brumeProduction,
  brumeStaging,
  beaubonProduction,
  melledProduction,
  eveDev,
];

export const services: readonly Service[] = [
  service(brumeProduction, "brume-api", "github", {
    domains: ["api.brume.dev"],
    dependsOn: ["Redis"],
  }),
  service(brumeProduction, "brume-worker", "github", { dependsOn: ["MongoDB"] }),
  service(brumeProduction, "MongoDB", "mongodb", { volume: "mongodb-data" }),
  service(brumeProduction, "Redis", "redis", { volume: "redis-data" }),
  service(brumeStaging, "brume-api", "github", {
    status: "deploying",
    domains: ["api.staging.brume.dev"],
    dependsOn: ["MongoDB"],
  }),
  service(brumeStaging, "MongoDB", "mongodb", { volume: "mongodb-data" }),
  service(beaubonProduction, "beaubon-api", "github", {
    domains: ["api.beaubon.fr"],
    dependsOn: ["PostgreSQL"],
  }),
  service(beaubonProduction, "beaubon-site", "static", { domains: ["beaubon.fr"] }),
  service(beaubonProduction, "PostgreSQL", "postgres", { volume: "postgres-data" }),
  service(melledProduction, "melled-web", "static", { domains: ["melled.app"] }),
  service(melledProduction, "melled-api", "github", {
    domains: ["api.melled.app"],
    dependsOn: ["MongoDB"],
  }),
  service(melledProduction, "MongoDB", "mongodb", { volume: "mongodb-data" }),
  service(eveDev, "eve", "github", { dependsOn: ["MongoDB", "Redis"] }),
  service(eveDev, "shim", "image", { status: "crashed", dependsOn: ["Redis"] }),
  service(eveDev, "tailscale-forwarder", "image", { volume: "tailscale-state" }),
  service(eveDev, "MongoDB", "mongodb", { volume: "mongodb-data" }),
  service(eveDev, "Redis", "redis", { volume: "redis-data" }),
];

export const repositories: readonly Repository[] = [
  "planchon/planchon",
  "brumecloud/brume",
  "planchon/beaubon",
  "planchon/eve",
  "planchon/melled",
  "planchon/coucou",
  "planchon/movictory",
  "planchon/ai-agent",
].map((fullName) => ({ fullName }));

// Every repository has the branches of the seeded environments; brume has a few more.
export function branchesOf(repository: string): string[] {
  const common = ["main", "dev", "staging"];
  return repository === "brumecloud/brume"
    ? [...common, "develop", "feat/canvas", "fix/log-stream"]
    : common;
}
