import type {
  ActivityEvent,
  ActivityPage,
  DeploymentId,
  Environment,
  EnvironmentId,
  Service,
  ServiceId,
} from "~/api/types";

import { decodeArray, decodeRecord, readNullable, readNumber, readString } from "./decode";

export type ActivityTone = "neutral" | "success" | "danger" | "info";

const DANGER: ReadonlySet<string> = new Set(["DeploymentFailed", "LoginFailed"]);

export function activityTone(type: string): ActivityTone {
  if (type === "DeploymentSucceeded") {
    return "success";
  }
  if (DANGER.has(type)) {
    return "danger";
  }
  return type.startsWith("Deployment") ? "info" : "neutral";
}

export function parseActivityPage(raw: unknown): ActivityPage {
  const record = decodeRecord(raw, "activity");
  return {
    events: decodeArray(record.events, parseActivityEvent, "activity events"),
    nextBefore: readNullable(record, "nextBefore", "activity", readNumber),
  };
}

function parseActivityEvent(raw: unknown): ActivityEvent {
  const record = decodeRecord(raw, "event");
  return {
    seq: readNumber(record, "seq", "event"),
    at: readString(record, "at", "event"),
    type: readString(record, "type", "event"),
    actor: readString(record, "actor", "event"),
    environmentId: readNullable(
      record,
      "environmentId",
      "event",
      readString,
    ) as EnvironmentId | null,
    serviceId: readNullable(record, "serviceId", "event", readString) as ServiceId | null,
    deploymentId: readNullable(record, "deploymentId", "event", readString) as DeploymentId | null,
    summary: readString(record, "summary", "event"),
  };
}

// Pages come newest first; the view reads like the log, oldest first. A page fetched again after
// new events may repeat some: each event appears once.
export function activityRows(pages: readonly ActivityPage[]): ActivityEvent[] {
  const bySeq = new Map<number, ActivityEvent>();
  for (const page of pages) {
    for (const event of page.events) {
      bySeq.set(event.seq, event);
    }
  }
  return [...bySeq.values()].toSorted((a, b) => a.seq - b.seq);
}

// What the event is about, by the names the panel still knows: `api · production`.
export function activityEntity(
  event: ActivityEvent,
  environments: readonly Environment[],
  services: readonly Service[],
): string | null {
  const service = services.find(({ id }) => id === event.serviceId);
  const environment = environments.find(
    ({ id }) => id === (service?.environmentId ?? event.environmentId),
  );
  if (event.serviceId) {
    return service && environment ? `${service.name} · ${environment.name}` : null;
  }
  return environment?.name ?? null;
}
