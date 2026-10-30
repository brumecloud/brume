import type { Commit, Deployment, DeploymentId, DeploymentStatus, ServiceId } from "~/api/types";

import {
  decodeArray,
  decodeRecord,
  readBoolean,
  readNullable,
  readOneOf,
  readString,
} from "./decode";

const STATUSES: readonly DeploymentStatus[] = [
  "building",
  "deploying",
  "active",
  "failed",
  "superseded",
];
const DATE_TIME_FORMAT = new Intl.DateTimeFormat("fr-FR", {
  dateStyle: "medium",
  timeStyle: "short",
});

export function parseDeployments(raw: unknown): Deployment[] {
  return decodeArray(raw, parseDeployment, "deployments");
}

function parseDeployment(raw: unknown): Deployment {
  const record = decodeRecord(raw, "deployment");
  return {
    id: readString(record, "id", "deployment") as DeploymentId,
    serviceId: readString(record, "serviceId", "deployment") as ServiceId,
    status: readOneOf(record, "status", STATUSES, "deployment"),
    createdAt: readString(record, "createdAt", "deployment"),
    author: readString(record, "author", "deployment"),
    commit: readNullable(record, "commit", "deployment", (r, key) => parseCommit(r[key])),
    image: readNullable(record, "image", "deployment", readString),
    rollbackable: readBoolean(record, "rollbackable", "deployment"),
  };
}

function parseCommit(raw: unknown): Commit {
  const record = decodeRecord(raw, "commit");
  return {
    sha: readString(record, "sha", "commit"),
    message: readString(record, "message", "commit"),
    branch: readString(record, "branch", "commit"),
  };
}

export function splitDeployments(deployments: readonly Deployment[]): {
  readonly active: Deployment | undefined;
  readonly others: Deployment[];
} {
  const newestFirst = deployments.toSorted(
    (a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt),
  );
  return {
    active: newestFirst.find((deployment) => deployment.status === "active"),
    others: newestFirst.filter((deployment) => deployment.status !== "active"),
  };
}

export function deploymentTitle(deployment: Deployment): string {
  return deployment.commit?.message.split("\n")[0] ?? deployment.image ?? "Déploiement";
}

export function formatDateTime(iso: string): string {
  return DATE_TIME_FORMAT.format(new Date(iso));
}
