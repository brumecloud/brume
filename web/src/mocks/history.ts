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

import { apps, environments, minutesAgo, projects, services } from "./seed";

const TEMPLATE_IMAGES = {
  postgres: "postgres:17.6",
  mongodb: "mongo:8.0.15",
  redis: "redis:8.2.2",
};

const HOUR = 60;
const DAY = 24 * HOUR;

// A few minutes apart from one service to the next, so the dates look like real ones.
const jitter = (service: Service) => services.indexOf(service) * 37;

// Newest first. While the newest one deploys, the previous one still serves the traffic. A failed
// deployment never served.
function commitStatus(service: Service, index: number, failed: boolean): DeploymentStatus {
  const deploying = service.status === "deploying";
  if (failed) {
    return "failed";
  }
  if (index === 0 && deploying) {
    return "deploying";
  }
  const commits = apps[service.name]?.commits ?? [];
  const live = commits.findIndex((commit, at) => at >= (deploying ? 1 : 0) && !commit.failed);
  return index === live ? "active" : "superseded";
}

function deploymentsOf(service: Service): Deployment[] {
  const app = apps[service.name];
  switch (service.kind) {
    case "github":
    case "static":
      return (app?.commits ?? []).map(({ sha, message, failed }, index) => ({
        id: `${service.id}-d${index}` as DeploymentId,
        serviceId: service.id,
        status: commitStatus(service, index, failed !== undefined),
        createdAt: minutesAgo(
          index === 0 && service.status === "deploying" ? 3 : index * 3 * DAY + jitter(service),
        ),
        author: "GitHub",
        commit: { sha, message, branch: "main" },
        image: null,
        // Only the artifacts of the latest revisions are kept.
        rollbackable: index < 4,
      }));
    case "image":
      return (app?.images ?? [service.name]).map((image, index) => ({
        id: `${service.id}-d${index}` as DeploymentId,
        serviceId: service.id,
        status: index === 0 ? "active" : "superseded",
        createdAt: minutesAgo((index * 12 + 2) * DAY + jitter(service)),
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
          createdAt: minutesAgo(30 * DAY + jitter(service)),
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
  return {
    id,
    level,
    ownerId,
    name,
    kind,
    value: kind === "secret" ? null : value,
    reference: false,
  };
};

const projectId = (name: string) => projects.find((project) => project.name === name)?.id ?? "";

const environmentId = (projectName: string, name: string) =>
  environments.find(
    (environment) => environment.projectId === projectId(projectName) && environment.name === name,
  )?.id ?? "";

const serviceId = (projectName: string, environmentName: string, name: string) =>
  services.find(
    (service) =>
      service.environmentId === environmentId(projectName, environmentName) &&
      service.name === name,
  )?.id ?? "";

// The variables of one level of a project: `[name, kind, value]`.
type Entries = readonly (readonly [string, VariableKind, string])[];

const ofProject = (projectName: string, entries: Entries) =>
  entries.map(([name, kind, value]) =>
    variable("project", projectId(projectName), name, kind, value),
  );

const ofEnvironment = (projectName: string, environmentName: string, entries: Entries) =>
  entries.map(([name, kind, value]) =>
    variable("environment", environmentId(projectName, environmentName), name, kind, value),
  );

const ofService = (
  projectName: string,
  environmentName: string,
  serviceName: string,
  entries: Entries,
) =>
  entries.map(([name, kind, value]) =>
    variable("service", serviceId(projectName, environmentName, serviceName), name, kind, value),
  );

const NORTHBEAN_DATABASE = "postgres://shop:Q7v2kLp9@postgresql:5432/shop";
const NORTHBEAN_REDIS = "redis://default:Hn4xT8wz@redis:6379";
const ROUTELANE_MONGODB = "mongodb://tracking:Vb6sR2qe@mongodb:27017/positions";
const ROUTELANE_REDIS = "redis://default:Kd9fW3mz@redis:6379";
const CLINICSLOT_DATABASE = "postgres://booking:Tz5hN8ca@postgresql:5432/booking";
const CLINICSLOT_REDIS = "redis://default:Pj2wY7ub@redis:6379";

// A service secret that points to a shared one by its name.
const reference = (
  projectName: string,
  environmentName: string,
  serviceName: string,
  name: string,
): Variable => {
  const ownerId = serviceId(projectName, environmentName, serviceName);
  return {
    id: `${ownerId}-${name}` as VariableId,
    level: "service",
    ownerId,
    name,
    kind: "secret",
    value: null,
    reference: true,
  };
};

export const variables: readonly Variable[] = [
  reference("northbean", "production", "shop-api", "SENTRY_DSN"),
  reference("northbean", "staging", "shop-api", "SENTRY_DSN"),
  ...ofProject("northbean", [
    ["LOG_LEVEL", "plain", "info"],
    ["SENTRY_DSN", "secret", "https://3f9c@o4512.ingest.sentry.io/1042"],
  ]),
  ...ofEnvironment("northbean", "production", [
    ["APP_URL", "plain", "https://northbean.coffee"],
    ["PUBLIC_API_URL", "build", "https://api.northbean.coffee"],
  ]),
  ...ofEnvironment("northbean", "staging", [
    ["APP_URL", "plain", "https://staging.northbean.coffee"],
    ["LOG_LEVEL", "plain", "debug"],
  ]),
  ...ofService("northbean", "production", "shop-api", [
    ["PORT", "plain", "3000"],
    ["DATABASE_URL", "secret", NORTHBEAN_DATABASE],
    ["REDIS_URL", "secret", NORTHBEAN_REDIS],
    ["CURRENCY", "plain", "EUR"],
    ["STRIPE_SECRET_KEY", "secret", "sk_demo_5f2a9c7e1b3d8a40"],
  ]),
  ...ofService("northbean", "production", "order-worker", [
    ["CONCURRENCY", "plain", "8"],
    ["DATABASE_URL", "secret", NORTHBEAN_DATABASE],
    ["REDIS_URL", "secret", NORTHBEAN_REDIS],
  ]),
  ...ofService("northbean", "staging", "shop-api", [
    ["PORT", "plain", "3000"],
    ["DATABASE_URL", "secret", "postgres://shop:Me3cX9rv@postgresql:5432/shop"],
  ]),

  ...ofProject("routelane", [["TZ", "plain", "Europe/Paris"]]),
  ...ofEnvironment("routelane", "production", [
    ["PUBLIC_API_URL", "build", "https://api.routelane.app"],
  ]),
  ...ofService("routelane", "production", "tracking-api", [
    ["PORT", "plain", "3000"],
    ["DATABASE_URL", "secret", "postgres://tracking:Lw4pG6dn@postgresql:5432/fleet"],
    ["MONGODB_URI", "secret", ROUTELANE_MONGODB],
    ["REDIS_URL", "secret", ROUTELANE_REDIS],
    ["GEOFENCE_ALERTS", "plain", "true"],
  ]),
  ...ofService("routelane", "production", "gps-ingest", [
    ["MONGODB_URI", "secret", ROUTELANE_MONGODB],
    ["REDIS_URL", "secret", ROUTELANE_REDIS],
    ["BATCH_SIZE", "plain", "1000"],
  ]),
  ...ofService("routelane", "production", "sms-gateway", [
    ["SMS_API_KEY", "secret", "a3f8c2e9d1b7406e"],
    ["REDIS_URL", "secret", ROUTELANE_REDIS],
    ["SENDER_ID", "plain", "ROUTELANE"],
  ]),

  ...ofEnvironment("deskmind", "production", [
    ["PUBLIC_API_URL", "build", "https://api.deskmind.app"],
  ]),
  ...ofService("deskmind", "production", "support-api", [
    ["PORT", "plain", "8000"],
    ["DATABASE_URL", "secret", "postgres://support:Fy8kD2tq@postgresql:5432/support"],
    ["REDIS_URL", "secret", "redis://default:Ns5vB9hx@redis:6379"],
    ["INFERENCE_URL", "plain", "http://inference:8000/v1"],
    ["HANDOVER_THRESHOLD", "plain", "0.6"],
  ]),
  ...ofService("deskmind", "production", "inference", [
    ["MODEL", "plain", "deskmind/support-7b"],
    ["GPU_MEMORY_UTILIZATION", "plain", "0.9"],
    ["HF_TOKEN", "secret", "c81f4e2a9d7b35f0"],
  ]),

  ...ofProject("tallybook", [["TZ", "plain", "Europe/Paris"]]),
  ...ofEnvironment("tallybook", "production", [
    ["PUBLIC_API_URL", "build", "https://api.tallybook.cloud"],
  ]),
  ...ofService("tallybook", "production", "billing-api", [
    ["PORT", "plain", "8080"],
    ["DATABASE_URL", "secret", "postgres://billing:Rc7mJ4zs@postgresql:5432/billing"],
    ["REDIS_URL", "secret", "redis://default:Gx3nQ8wf@redis:6379"],
    ["PDF_RENDERER_URL", "plain", "http://pdf-renderer:3000"],
    ["SMTP_PASSWORD", "secret", "Ue6tK1pa9Lzr"],
  ]),
  ...ofService("tallybook", "production", "pdf-renderer", [
    ["API_TIMEOUT", "plain", "30s"],
    ["CHROMIUM_MAX_QUEUE_SIZE", "plain", "20"],
  ]),
  ...ofService("tallybook", "staging", "billing-api", [
    ["PORT", "plain", "8080"],
    ["DATABASE_URL", "secret", "postgres://billing:Ha2xV5ke@postgresql:5432/billing"],
  ]),

  ...ofEnvironment("clinicslot", "production", [
    ["PUBLIC_API_URL", "build", "https://api.clinicslot.app"],
  ]),
  ...ofService("clinicslot", "production", "booking-api", [
    ["PORT", "plain", "8000"],
    ["DATABASE_URL", "secret", CLINICSLOT_DATABASE],
    ["REDIS_URL", "secret", CLINICSLOT_REDIS],
    ["DJANGO_SECRET_KEY", "secret", "w9e2r7t4y1u8i5o3p6a0s2d7"],
    ["TIME_ZONE", "plain", "Europe/Paris"],
  ]),
  ...ofService("clinicslot", "production", "reminders-worker", [
    ["DATABASE_URL", "secret", CLINICSLOT_DATABASE],
    ["REDIS_URL", "secret", CLINICSLOT_REDIS],
    ["SMS_API_KEY", "secret", "e5b9d3a7c1f8264b"],
    ["REMINDER_HOURS", "plain", "24,2"],
  ]),
];
