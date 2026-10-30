import type { LogLine } from "~/api/types";
import { formatLogTime } from "~/helpers/logs";

import { AnsiText } from "./ansi-text";
import { LogDetails } from "./log-details";
import { LEVEL_COLORS } from "./log-levels";

interface LogRowProps {
  readonly line: LogLine;
  // Omitted in the logs of a single deployment.
  readonly serviceName?: string | undefined;
  readonly open: boolean;
  readonly onToggle: () => void;
}

// Reference: infra/screens/03f-project-logs.png. A click opens the whole line as JSON.
export function LogRow({ line, serviceName, open, onToggle }: LogRowProps) {
  return (
    <div>
      <button
        type="button"
        aria-expanded={open}
        className="flex w-full gap-3 px-4 py-1 text-start font-mono text-xs leading-5 hover:bg-accent aria-expanded:bg-accent"
        onClick={onToggle}
      >
        <span
          aria-hidden="true"
          className={`w-0.5 shrink-0 rounded-full ${LEVEL_COLORS[line.level]}`}
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
      </button>
      {open && <LogDetails line={line} serviceName={serviceName} />}
    </div>
  );
}
