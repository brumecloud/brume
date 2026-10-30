import type {
  EnvironmentId,
  Service,
  ServiceId,
  ServiceKind,
  ServiceStatus,
  Volume,
} from "~/api/types";

import { decodeArray, decodeRecord, readOneOf, readString, readStringArray } from "./decode";
import { isId } from "./ids";

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

export function parseServiceId(raw: string): ServiceId | undefined {
  return isId(raw) ? (raw as ServiceId) : undefined;
}

export function isDatabase(kind: ServiceKind): boolean {
  return kind === "postgres" || kind === "mongodb" || kind === "redis";
}

const HOSTNAME = /^(?=.{1,253}$)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,63}$/i;
const NUMBER_FORMAT = new Intl.NumberFormat("fr-FR");

// Domains come from the API: link to them only over https, and only when they are hostnames.
export function domainUrl(domain: string): string | undefined {
  return HOSTNAME.test(domain) ? new URL(`https://${domain}`).href : undefined;
}

export function formatCpu(cpu: number): string {
  return `${NUMBER_FORMAT.format(cpu)} vCPU`;
}

export function formatMemory(memoryMb: number): string {
  return memoryMb >= 1024 ? `${NUMBER_FORMAT.format(memoryMb / 1024)} Go` : `${memoryMb} Mo`;
}

export function formatLimits(cpu: number, memoryMb: number): string {
  return `${formatCpu(cpu)} · ${formatMemory(memoryMb)}`;
}

export function serviceNames(services: readonly Service[]): Map<string, string> {
  return new Map(services.map((service) => [service.id, service.name]));
}
