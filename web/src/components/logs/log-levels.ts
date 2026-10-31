import type { LogLevel } from "~/api/types";

// One color per level, shared by the rows and the histogram.
export const LEVEL_COLORS: Record<LogLevel, string> = {
  info: "bg-info/70",
  warn: "bg-warning",
  error: "bg-destructive",
};

// Bottom to top in a histogram bar: errors stay visible at the top.
export const STACKED_LEVELS: readonly LogLevel[] = ["info", "warn", "error"];
