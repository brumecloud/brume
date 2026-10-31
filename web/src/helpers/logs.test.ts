import { describe, expect, it } from "vite-plus/test";

import type { LogLevel, LogLine, ServiceId } from "~/api/types";

import { DecodeError } from "./decode";
import {
  filterLogs,
  formatLogTime,
  isNearBottom,
  jsonTokens,
  logCountLabel,
  logHistogram,
  logJson,
  logsAsText,
  mergeLogLines,
  parseLogLines,
  parseLogsSearch,
} from "./logs";

const line = (
  id: string,
  message: string,
  level: LogLevel = "info",
  serviceId = "s-api",
  timestamp = "2026-10-28T22:48:00Z",
): LogLine => ({ id, timestamp, serviceId: serviceId as ServiceId, level, message });

describe("parseLogsSearch", () => {
  it("keeps known levels, a bounded search and a valid service id", () => {
    expect(parseLogsSearch({ level: "warn", q: "x".repeat(300), service: "s-api" })).toEqual({
      level: "warn",
      q: "x".repeat(200),
      service: "s-api",
    });
  });

  it("drops anything else", () => {
    expect(parseLogsSearch({ level: "debug", q: "", service: "../x" })).toEqual({});
  });
});

describe("parseLogLines", () => {
  const raw = {
    id: "l-1",
    timestamp: "2026-10-28T22:48:00Z",
    serviceId: "s-api",
    level: "warn",
    message: "slow query",
  };

  it("decodes a batch of lines", () => {
    expect(parseLogLines([raw])).toEqual([raw]);
  });

  it("rejects an unknown level", () => {
    expect(() => parseLogLines([{ ...raw, level: "trace" }])).toThrow(DecodeError);
  });
});

describe("filterLogs", () => {
  const lines = [
    line("1", "GET /health 200"),
    line("2", "Slow query on users", "warn"),
    line("3", "Connection refused", "error", "s-worker"),
  ];
  const ids = (search: Parameters<typeof filterLogs>[1]) =>
    filterLogs(lines, search).map(({ id }) => id);

  it("keeps everything without filter", () => {
    expect(ids({})).toEqual(["1", "2", "3"]);
  });

  it("keeps warnings and errors from the warn level", () => {
    expect(ids({ level: "warn" })).toEqual(["2", "3"]);
    expect(ids({ level: "error" })).toEqual(["3"]);
  });

  it("searches the message, ignoring case", () => {
    expect(ids({ q: "QUERY" })).toEqual(["2"]);
  });

  it("filters on a service", () => {
    expect(ids({ service: "s-worker" })).toEqual(["3"]);
  });
});

describe("mergeLogLines", () => {
  it("appends and keeps only the most recent lines", () => {
    const merged = mergeLogLines([line("1", ""), line("2", "")], [line("3", "")], 2);

    expect(merged.map(({ id }) => id)).toEqual(["2", "3"]);
  });

  it("skips lines already received, as when the stream sends its backlog again", () => {
    const merged = mergeLogLines(
      [line("1", ""), line("2", "")],
      [line("2", ""), line("3", "")],
      10,
    );

    expect(merged.map(({ id }) => id)).toEqual(["1", "2", "3"]);
  });
});

describe("formatLogTime", () => {
  it("formats a timestamp as date and time to the second", () => {
    expect(formatLogTime("2026-10-28T12:48:11Z")).toMatch(/^2026-10-28 \d{2}:48:11$/);
  });
});

const heights = (lines: readonly LogLine[], buckets: number) =>
  logHistogram(lines, buckets).map((bar) => bar.height);

describe("logHistogram", () => {
  it("counts the lines per time bucket, scaled against the busiest", () => {
    const lines = [
      line("1", "", "info", "s", "2026-10-28T22:00:00Z"),
      line("2", "", "info", "s", "2026-10-28T22:00:59Z"),
      line("3", "", "info", "s", "2026-10-28T22:01:00Z"),
    ];

    expect(heights(lines, 2)).toEqual([50, 100]);
  });

  it("splits each bucket by level", () => {
    const lines = [line("1", "", "info"), line("2", "", "error"), line("3", "", "error")];

    expect(logHistogram(lines, 2)).toEqual([
      { height: 100, counts: { info: 1, warn: 0, error: 2 } },
      { height: 0, counts: { info: 0, warn: 0, error: 0 } },
    ]);
  });

  it("handles a large buffer", () => {
    const lines = Array.from({ length: 200_000 }, (_, index) =>
      line(String(index), "", "info", "s", new Date(index * 1000).toISOString()),
    );

    expect(logHistogram(lines, 4).map((bar) => bar.counts.info)).toEqual([
      50_000, 50_000, 50_000, 50_000,
    ]);
  });

  it("returns no bucket without lines", () => {
    expect(logHistogram([], 10)).toEqual([]);
  });
});

describe("isNearBottom", () => {
  it("is true when the end of the list is in view", () => {
    expect(isNearBottom(970, 1000, 2000)).toBe(true);
    expect(isNearBottom(0, 1000, 800)).toBe(true);
  });

  it("is false once the admin scrolled up", () => {
    expect(isNearBottom(500, 1000, 2000)).toBe(false);
  });
});

describe("logCountLabel", () => {
  it.each([
    [0, "Aucune ligne"],
    [1, "1 ligne"],
    [500, "500 lignes"],
  ])("%i → %s", (count, label) => {
    expect(logCountLabel(count)).toBe(label);
  });
});

describe("logsAsText", () => {
  it("writes one line per log, timestamp first, escape codes removed", () => {
    const lines = [
      line("1", "\u001B[32mINFO\u001B[0m started", "info", "s", "2026-10-28T22:48:00Z"),
      line("2", "done", "info", "s", "2026-10-28T22:48:01Z"),
    ];

    expect(logsAsText(lines)).toBe(
      "2026-10-28T22:48:00Z INFO started\n2026-10-28T22:48:01Z done\n",
    );
  });
});

describe("logJson", () => {
  it("shows a JSON message as data", () => {
    const json = logJson(line("1", '{"t":{"$date":"2026-10-28"},"msg":"ok"}', "info"), "MongoDB");

    expect(JSON.parse(json)).toEqual({
      timestamp: "2026-10-28T22:48:00Z",
      level: "info",
      service: "MongoDB",
      data: { t: { $date: "2026-10-28" }, msg: "ok" },
    });
  });

  it("keeps a text message as text, without its ANSI codes", () => {
    const json = logJson(line("1", "\u001B[31mERROR\u001B[0m {not json"));

    expect(JSON.parse(json)).toEqual({
      timestamp: "2026-10-28T22:48:00Z",
      level: "info",
      message: "ERROR {not json",
    });
  });
});

describe("jsonTokens", () => {
  it("tells keys, strings, numbers and literals apart", () => {
    expect(jsonTokens('{"a": "x:1", "b": -2.5e3, "c": [true, null]}')).toEqual([
      { text: "{", kind: "plain" },
      { text: '"a"', kind: "key" },
      { text: ":", kind: "plain" },
      { text: " ", kind: "plain" },
      { text: '"x:1"', kind: "string" },
      { text: ", ", kind: "plain" },
      { text: '"b"', kind: "key" },
      { text: ":", kind: "plain" },
      { text: " ", kind: "plain" },
      { text: "-2.5e3", kind: "number" },
      { text: ", ", kind: "plain" },
      { text: '"c"', kind: "key" },
      { text: ":", kind: "plain" },
      { text: " [", kind: "plain" },
      { text: "true", kind: "literal" },
      { text: ", ", kind: "plain" },
      { text: "null", kind: "literal" },
      { text: "]}", kind: "plain" },
    ]);
  });

  it("keeps escaped quotes inside their string", () => {
    expect(jsonTokens('"<script>\\"1\\"</script>"')).toEqual([
      { text: '"<script>\\"1\\"</script>"', kind: "string" },
    ]);
  });
});
