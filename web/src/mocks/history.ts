import type {
  Deployment,
  DeploymentId,
  DeploymentStatus,
  Service,
  Variable,
  VariableId,
  VariableKind,
  VariableLevel,
} from "~/api/types";

import { environments, projects, services } from "./seed";

const COMMITS = [
  { sha: "d63eac96f1b2", message: "feat(web): add payload preview dialog on node cards" },
  { sha: "a41c0e7d9b30", message: "fix(api): converge engine indexes when a definition changes" },
  { sha: "7f2b9c1e5d44", message: "refacto code and docs" },
  { sha: "c90d4a2f1e87", message: "chore: bump dependencies" },
  { sha: "3e8f6b0a7c12", message: "feat(worker): retry failed jobs with backoff" },
] as const;

const IMAGES: Record<string, readonly string[]> = {
  shim: ["ghcr.io/planchon/shim:1.4.0", "ghcr.io/planchon/shim:1.3.2"],
  "tailscale-forwarder": ["ghcr.io/tailscale/tailscale:v1.88.3"],
};

const TEMPLATE_IMAGES = {
  postgres: "postgres:17.6",
  mongodb: "mongo:8.0.15",
  redis: "redis:8.2.2",
};

const DAY_MS = 86_400_000;
const NOW = Date.parse("2026-10-29T21:00:00Z");

const daysAgo = (days: number) => new Date(NOW - days * DAY_MS).toISOString();

// Newest first. While the newest one deploys, the previous one still serves the traffic.
function commitStatus(service: Service, index: number): DeploymentStatus {
  if (index === 0) {
    return service.status === "deploying" ? "deploying" : "active";
  }
  if (index === 1 && service.status === "deploying") {
    return "active";
  }
  return index === 3 ? "failed" : "superseded";
}

function deploymentsOf(service: Service): Deployment[] {
  switch (service.kind) {
    case "github":
    case "static":
      return COMMITS.map(({ sha, message }, index) => ({
        id: `${service.id}-d${index}` as DeploymentId,
        serviceId: service.id,
        status: commitStatus(service, index),
        createdAt: daysAgo(index * 9 + 1),
        author: "GitHub",
        commit: { sha, message, branch: "main" },
        image: null,
        // Only the artifacts of the latest revisions are kept.
        rollbackable: index < 4,
      }));
    case "image":
      return (IMAGES[service.name] ?? [service.name]).map((image, index) => ({
        id: `${service.id}-d${index}` as DeploymentId,
        serviceId: service.id,
        status: index === 0 ? "active" : "superseded",
        createdAt: daysAgo(index * 14 + 3),
        author: "admin",
        commit: null,
        image,
        rollbackable: true,
      }));
    default:
      return [
        {
          id: `${service.id}-d0` as DeploymentId,
          serviceId: service.id,
          status: "active",
          createdAt: daysAgo(30),
          author: "admin",
          commit: null,
          image: TEMPLATE_IMAGES[service.kind],
          rollbackable: true,
        },
      ];
  }
}

export const deployments: readonly Deployment[] = services.flatMap(deploymentsOf);

// Secret values live apart: the API only hands them out through a reveal.
export const secretValues = new Map<string, string>();

const variable = (
  level: VariableLevel,
  ownerId: string,
  name: string,
  kind: VariableKind,
  value: string,
): Variable => {
  const id = `${ownerId}-${name}` as VariableId;
  if (kind === "secret") {
    secretValues.set(id, value);
  }
  return { id, level, ownerId, name, kind, value: kind === "secret" ? null : value };
};

const serviceId = (environmentIndex: number, name: string) =>
  services.find(
    (service) =>
      service.environmentId === environments[environmentIndex]?.id && service.name === name,
  )?.id ?? "";

const [brume, beaubon] = projects;
const [brumeProduction, brumeStaging] = environments;

export const variables: readonly Variable[] = [
  variable("project", brume?.id ?? "", "LOG_LEVEL", "plain", "info"),
  variable(
    "project",
    brume?.id ?? "",
    "SENTRY_DSN",
    "secret",
    "https://4f1c@o12.ingest.sentry.io/77",
  ),
  variable("environment", brumeProduction?.id ?? "", "APP_URL", "plain", "https://brume.dev"),
  variable(
    "environment",
    brumeProduction?.id ?? "",
    "PUBLIC_API_URL",
    "build",
    "https://api.brume.dev",
  ),
  variable("environment", brumeStaging?.id ?? "", "APP_URL", "plain", "https://staging.brume.dev"),
  variable("environment", brumeStaging?.id ?? "", "LOG_LEVEL", "plain", "debug"),
  variable("service", serviceId(0, "brume-api"), "PORT", "plain", "3000"),
  variable(
    "service",
    serviceId(0, "brume-api"),
    "REDIS_URL",
    "secret",
    "redis://app:q8Hk2v@redis:6379",
  ),
  variable(
    "service",
    serviceId(0, "brume-api"),
    "OPENROUTER_API_KEY",
    "secret",
    "sk-or-v1-9f8e7d6c5b4a",
  ),
  variable("service", serviceId(0, "brume-api"), "R2_ACCOUNT_ID", "plain", "4b1d0e2c9a8f"),
  variable("service", serviceId(0, "brume-api"), "R2_SECRET_KEY", "secret", "r2-secret-1a2b3c4d"),
  variable("service", serviceId(0, "brume-worker"), "CONCURRENCY", "plain", "8"),
  variable(
    "service",
    serviceId(0, "brume-worker"),
    "MONGODB_URI",
    "secret",
    "mongodb://app:Zx91@mongodb:27017/brume",
  ),
  variable("service", serviceId(1, "brume-api"), "PORT", "plain", "3000"),
  variable(
    "service",
    serviceId(2, "beaubon-api"),
    "DATABASE_URL",
    "secret",
    "postgres://app:Lp4m@postgresql:5432/beaubon",
  ),
  variable(
    "service",
    serviceId(2, "beaubon-site"),
    "PUBLIC_API_URL",
    "build",
    "https://api.beaubon.fr",
  ),
  variable("project", beaubon?.id ?? "", "TZ", "plain", "Europe/Paris"),
];
