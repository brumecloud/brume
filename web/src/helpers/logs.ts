import type { LogLevel, LogLine, ServiceId } from "~/api/types";

import { decodeArray, decodeRecord, readOneOf, readString } from "./decode";
import { isId } from "./ids";

const LEVELS: readonly LogLevel[] = ["info", "warn", "error"];
const MAX_QUERY_LENGTH = 200;
// Swedish dates read as ISO: 2026-10-28 23:48:11, in the admin's time zone.
const TIME_FORMAT = new Intl.DateTimeFormat("sv-SE", {
  dateStyle: "short",
  timeStyle: "medium",
});

// Absent values mean every level, every service, no search.
export interface LogsSearch {
  readonly level?: "warn" | "error" | undefined;
  readonly q?: string | undefined;
  readonly service?: string | undefined;
}

export function parseLogsSearch(raw: Record<string, unknown>): LogsSearch {
  return {
    ...(raw.level === "warn" || raw.level === "error" ? { level: raw.level } : {}),
    ...(typeof raw.q === "string" && raw.q.trim() ? { q: raw.q.slice(0, MAX_QUERY_LENGTH) } : {}),
    ...(typeof raw.service === "string" && isId(raw.service) ? { service: raw.service } : {}),
  };
}

export function parseLogLines(raw: unknown): LogLine[] {
  return decodeArray(raw, parseLogLine, "log lines");
}

function parseLogLine(raw: unknown): LogLine {
  const record = decodeRecord(raw, "log line");
  return {
    id: readString(record, "id", "log line"),
    timestamp: readString(record, "timestamp", "log line"),
    serviceId: readString(record, "serviceId", "log line") as ServiceId,
    level: readOneOf(record, "level", LEVELS, "log line"),
    message: readString(record, "message", "log line"),
  };
}

export function filterLogs(lines: readonly LogLine[], search: LogsSearch): LogLine[] {
  const query = search.q?.toLowerCase();
  return lines.filter(
    (line) =>
      (search.level !== "warn" || line.level !== "info") &&
      (search.level !== "error" || line.level === "error") &&
      (!search.service || line.serviceId === search.service) &&
      (!query || line.message.toLowerCase().includes(query)),
  );
}

// Only the latest lines stay in memory: a chatty service cannot fill the tab. A reconnected
// stream sends its backlog again: lines already received are skipped.
export function mergeLogLines(
  lines: readonly LogLine[],
  incoming: readonly LogLine[],
  max: number,
): LogLine[] {
  const known = new Set(lines.map((line) => line.id));
  return [...lines, ...incoming.filter((line) => !known.has(line.id))].slice(-max);
}

export function formatLogTime(iso: string): string {
  return TIME_FORMAT.format(new Date(iso));
}

// Lines per time bucket, from the first line to the last one. A loop rather than
// Math.min(...times): tens of thousands of arguments would overflow the stack.
export function logHistogram(lines: readonly LogLine[], bucketCount: number): number[] {
  if (lines.length === 0) {
    return [];
  }
  const times = lines.map((line) => Date.parse(line.timestamp));
  let first = Infinity;
  let last = -Infinity;
  for (const time of times) {
    first = Math.min(first, time);
    last = Math.max(last, time);
  }
  const span = last - first;
  const buckets = Array.from({ length: bucketCount }, () => 0);
  for (const time of times) {
    const index =
      span === 0 ? 0 : Math.min(Math.floor(((time - first) / span) * bucketCount), bucketCount - 1);
    buckets[index] = (buckets[index] ?? 0) + 1;
  }
  return buckets;
}

const BOTTOM_THRESHOLD_PX = 32;

export function isNearBottom(
  scrollTop: number,
  clientHeight: number,
  scrollHeight: number,
): boolean {
  return scrollHeight - scrollTop - clientHeight <= BOTTOM_THRESHOLD_PX;
}

export function barHeights(counts: readonly number[]): number[] {
  const busiest = Math.max(0, ...counts);
  return counts.map((count) => (busiest === 0 ? 0 : (count / busiest) * 100));
}

export function logCountLabel(count: number): string {
  if (count === 0) {
    return "Aucune ligne";
  }
  return `${count} ${count > 1 ? "lignes" : "ligne"}`;
}
