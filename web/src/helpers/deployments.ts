import type {
  Commit,
  Deployment,
  DeploymentId,
  DeploymentLogStream,
  DeploymentStatus,
  Revision,
  RevisionVariable,
  ServiceId,
  VariableKind,
  VariableLevel,
} from "~/api/types";

import {
  decodeArray,
  decodeRecord,
  readBoolean,
  readNullable,
  readOneOf,
  readString,
} from "./decode";
import { isId } from "./ids";

const STATUSES: readonly DeploymentStatus[] = [
  "building",
  "deploying",
  "active",
  "failed",
  "superseded",
];
const VARIABLE_KINDS: readonly VariableKind[] = ["plain", "build", "secret"];
const VARIABLE_LEVELS: readonly VariableLevel[] = ["project", "environment", "service"];
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

export function parseRevision(raw: unknown): Revision {
  const record = decodeRecord(raw, "revision");
  return {
    digest: readString(record, "digest", "revision"),
    builder: readNullable(record, "builder", "revision", readString),
    startCommand: readNullable(record, "startCommand", "revision", readString),
    variables: decodeArray(record.variables, parseRevisionVariable, "revision.variables"),
  };
}

function parseRevisionVariable(raw: unknown): RevisionVariable {
  const record = decodeRecord(raw, "revision variable");
  return {
    name: readString(record, "name", "revision variable"),
    kind: readOneOf(record, "kind", VARIABLE_KINDS, "revision variable"),
    value: readNullable(record, "value", "revision variable", readString),
    origin: readOneOf(record, "origin", VARIABLE_LEVELS, "revision variable"),
  };
}

export interface DeploymentSearch {
  // Absent: the details of the revision.
  readonly tab?: DeploymentLogStream;
}

export function parseDeploymentSearch(raw: Record<string, unknown>): DeploymentSearch {
  return raw.tab === "build" || raw.tab === "deploy" ? { tab: raw.tab } : {};
}

export function parseDeploymentId(raw: string): DeploymentId | undefined {
  return isId(raw) ? (raw as DeploymentId) : undefined;
}
