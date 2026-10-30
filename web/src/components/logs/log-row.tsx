import type { LogLevel, LogLine } from "~/api/types";
import { formatLogTime } from "~/helpers/logs";

import { AnsiText } from "./ansi-text";

const LEVEL_BARS: Record<LogLevel, string> = {
  info: "bg-info/60",
  warn: "bg-warning",
  error: "bg-destructive",
};

interface LogRowProps {
  readonly line: LogLine;
  // Omitted in the logs of a single deployment.
  readonly serviceName?: string | undefined;
}

// Reference: infra/screens/03f-project-logs.png.
export function LogRow({ line, serviceName }: LogRowProps) {
  return (
    <div className="flex gap-3 px-4 py-1 font-mono text-xs leading-5 hover:bg-accent">
      <span
        aria-hidden="true"
        className={`w-0.5 shrink-0 rounded-full ${LEVEL_BARS[line.level]}`}
      />
      <time dateTime={line.timestamp} className="w-36 shrink-0 text-muted-foreground">
        {formatLogTime(line.timestamp)}
      </time>
      {serviceName !== undefined && (
        <span className="w-32 shrink-0 truncate text-muted-foreground">{serviceName}</span>
      )}
      <span className="min-w-0 flex-1 whitespace-pre-wrap break-words">
        <AnsiText text={line.message} />
      </span>
    </div>
  );
}
