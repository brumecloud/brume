import type { ActivityEvent, Deployment } from "~/api/types";

import { deployments } from "./history";
import { environments, projects, services } from "./seed";

// An event of the mocked log, with the project it belongs to: the API filters on it.
export interface StoredEvent {
  readonly projectId: string;
  readonly event: ActivityEvent;
}

type Draft = Omit<StoredEvent, "event"> & { readonly event: Omit<ActivityEvent, "seq"> };

const MINUTE_MS = 60_000;

const at = (iso: string, minutes: number) =>
  new Date(Date.parse(iso) + minutes * MINUTE_MS).toISOString();

export function deploymentEvent(
  deployment: Deployment,
  serviceName: string,
  environmentName: string,
): Pick<ActivityEvent, "type" | "summary"> {
  switch (deployment.status) {
    case "failed":
      return {
        type: "DeploymentFailed",
        summary: `Deployment of ${serviceName} failed in ${environmentName}`,
      };
    case "building":
    case "deploying":
      return {
        type: "DeploymentStarted",
        summary: `Started deploying ${serviceName} in ${environmentName}`,
      };
    default:
      return {
        type: "DeploymentSucceeded",
        summary: `Deployed ${serviceName} in ${environmentName}`,
      };
  }
}

// The log behind the seeded projects: each project with its environments and services, then
// every deployment of their history, numbered in time order.
export function seedEvents(): StoredEvent[] {
  const drafts: Draft[] = [];
  const base = { actor: "admin", environmentId: null, serviceId: null, deploymentId: null };
  for (const project of projects) {
    const own = environments.filter((environment) => environment.projectId === project.id);
    const ownServices = services.filter((service) =>
      own.some((environment) => environment.id === service.environmentId),
    );
    const history = deployments.filter((deployment) =>
      ownServices.some((service) => service.id === deployment.serviceId),
    );
    const first = history.map(({ createdAt }) => createdAt).toSorted()[0] ?? project.updatedAt;
    const start = at(first, -60);
    const add = (event: Draft["event"]) => drafts.push({ projectId: project.id, event });
    add({ ...base, at: start, type: "ProjectCreated", summary: `Created project ${project.name}` });
    for (const environment of own) {
      add({
        ...base,
        at: at(start, 1),
        type: "EnvironmentCreated",
        environmentId: environment.id,
        summary: `Created environment ${environment.name}`,
      });
    }
    for (const service of ownServices) {
      const environment = own.find(({ id }) => id === service.environmentId);
      add({
        ...base,
        at: at(start, 2),
        type: "ServiceCreated",
        environmentId: service.environmentId,
        serviceId: service.id,
        summary: `Created service ${service.name} in ${environment?.name ?? "?"}`,
      });
    }
    for (const deployment of history) {
      const service = ownServices.find(({ id }) => id === deployment.serviceId);
      const environment = own.find(({ id }) => id === service?.environmentId);
      add({
        at: deployment.createdAt,
        actor: deployment.author,
        environmentId: environment?.id ?? null,
        serviceId: deployment.serviceId,
        deploymentId: deployment.id,
        ...deploymentEvent(deployment, service?.name ?? "?", environment?.name ?? "?"),
      });
    }
  }
  const ordered = drafts.toSorted((a, b) => a.event.at.localeCompare(b.event.at));
  return ordered.map(({ projectId, event }, index) => ({
    projectId,
    event: Object.assign({ seq: index + 1 }, event),
  }));
}
