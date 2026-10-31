import type { LogLevel, LogLine, ServiceId } from "~/api/types";

import { parseAnsi } from "./ansi";
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
type LevelCounts = Record<LogLevel, number>;

export interface HistogramBar {
  // In percent of the busiest bucket.
  readonly height: number;
  readonly counts: Readonly<LevelCounts>;
}

// The visible lines over time, each bucket split by level.
export function logHistogram(lines: readonly LogLine[], bucketCount: number): HistogramBar[] {
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
  const buckets = Array.from({ length: bucketCount }, (): LevelCounts => ({
    info: 0,
    warn: 0,
    error: 0,
  }));
  for (const [index, line] of lines.entries()) {
    const time = times[index] ?? first;
    const bucket =
      buckets[
        span === 0
          ? 0
          : Math.min(Math.floor(((time - first) / span) * bucketCount), bucketCount - 1)
      ];
    if (bucket) {
      bucket[line.level] += 1;
    }
  }
  const totals = buckets.map((counts) => counts.info + counts.warn + counts.error);
  const busiest = Math.max(0, ...totals);
  return buckets.map((counts, index) => ({
    height: busiest === 0 ? 0 : ((totals[index] ?? 0) / busiest) * 100,
    counts,
  }));
}

const BOTTOM_THRESHOLD_PX = 32;

export function isNearBottom(
  scrollTop: number,
  clientHeight: number,
  scrollHeight: number,
): boolean {
  return scrollHeight - scrollTop - clientHeight <= BOTTOM_THRESHOLD_PX;
}

export function logCountLabel(count: number): string {
  if (count === 0) {
    return "Aucune ligne";
  }
  return `${count} ${count > 1 ? "lignes" : "ligne"}`;
}

function plainText(message: string): string {
  return parseAnsi(message)
    .map((segment) => segment.text)
    .join("");
}

// A downloadable text file: plain lines, the ANSI codes are gone.
export function logsAsText(lines: readonly LogLine[]): string {
  return lines.map((line) => `${line.timestamp} ${plainText(line.message)}\n`).join("");
}

// The whole line as indented JSON. A message that is itself JSON shows as data, not as an
// escaped string.
export function logJson(line: LogLine, serviceName?: string): string {
  const message = plainText(line.message);
  const data = parseJsonMessage(message);
  return JSON.stringify(
    {
      timestamp: line.timestamp,
      level: line.level,
      ...(serviceName ? { service: serviceName } : {}),
      ...(data === undefined ? { message } : { data }),
    },
    null,
    2,
  );
}

function parseJsonMessage(message: string): unknown {
  const text = message.trim();
  if (!text.startsWith("{") && !text.startsWith("[")) {
    return undefined;
  }
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return undefined;
  }
}

export type JsonTokenKind = "key" | "string" | "number" | "literal" | "plain";

export interface JsonToken {
  readonly text: string;
  readonly kind: JsonTokenKind;
}

// A whole string first, so the digits and words inside it stay part of it.
const JSON_TOKEN =
  /("(?:[^"\\]|\\.)*")(\s*:)?|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|\b(?:true|false|null)\b/g;

// Splits JSON text into colored tokens. Rendered as text: nothing here becomes markup.
export function jsonTokens(json: string): JsonToken[] {
  const tokens: JsonToken[] = [];
  let last = 0;
  for (const match of json.matchAll(JSON_TOKEN)) {
    const [text, quoted, colon] = match;
    if (match.index > last) {
      tokens.push({ text: json.slice(last, match.index), kind: "plain" });
    }
    if (quoted === undefined) {
      tokens.push({ text, kind: /\d/.test(text) ? "number" : "literal" });
    } else {
      tokens.push({ text: quoted, kind: colon ? "key" : "string" });
      if (colon) {
        tokens.push({ text: colon, kind: "plain" });
      }
    }
    last = match.index + text.length;
  }
  if (last < json.length) {
    tokens.push({ text: json.slice(last), kind: "plain" });
  }
  return tokens;
}
