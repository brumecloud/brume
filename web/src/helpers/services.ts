import type {
  EnvironmentId,
  Service,
  ServiceId,
  ServiceKind,
  ServiceStatus,
  Volume,
} from "~/api/types";

import { decodeArray, decodeRecord, readOneOf, readString, readStringArray } from "./decode";

const KINDS: readonly ServiceKind[] = ["github", "image", "static", "postgres", "mongodb", "redis"];
const STATUSES: readonly ServiceStatus[] = ["online", "deploying", "crashed", "stopped"];

export function parseServices(raw: unknown): Service[] {
  return decodeArray(raw, parseService, "services");
}

function parseService(raw: unknown): Service {
  const record = decodeRecord(raw, "service");
  return {
    id: readString(record, "id", "service") as ServiceId,
    environmentId: readString(record, "environmentId", "service") as EnvironmentId,
    name: readString(record, "name", "service"),
    slug: readString(record, "slug", "service"),
    kind: readOneOf(record, "kind", KINDS, "service"),
    status: readOneOf(record, "status", STATUSES, "service"),
    domains: readStringArray(record, "domains", "service"),
    volume: record.volume === null ? null : parseVolume(record.volume),
    dependencies: readStringArray(record, "dependencies", "service") as ServiceId[],
  };
}

function parseVolume(raw: unknown): Volume {
  const record = decodeRecord(raw, "volume");
  return {
    name: readString(record, "name", "volume"),
    mountPath: readString(record, "mountPath", "volume"),
  };
}
