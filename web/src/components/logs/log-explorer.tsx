import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useEffect, useState } from "react";

import { subscribeToLogs } from "~/api/logs";
import { environmentServices } from "~/api/services";
import type { EnvironmentId, LogLine } from "~/api/types";
import { type LogsSearch, mergeLogLines, visibleLogs } from "~/helpers/logs";
import { serviceNames } from "~/helpers/services";

import { LogHistogram } from "./log-histogram";
import { LogList } from "./log-list";
import { LogToolbar } from "./log-toolbar";

// The tab keeps the latest 5000 lines and shows the latest 500 that match the filters.
const KEPT_LINES = 5000;
const SHOWN_LINES = 500;

interface LogExplorerProps {
  readonly environmentId: EnvironmentId;
  readonly search: LogsSearch;
}

// Reference: infra/screens/03f-project-logs.png.
export function LogExplorer({ environmentId, search }: LogExplorerProps) {
  const { data: services } = useLiveSuspenseQuery({ query: environmentServices(environmentId) });
  const [lines, setLines] = useState<LogLine[]>([]);
  const [paused, setPaused] = useState(false);

  // oxlint-disable-next-line brume/no-unreasoned-effect -- opens the log SSE stream and closes it on unmount or pause
  useEffect(() => {
    if (paused) {
      return undefined;
    }
    return subscribeToLogs(environmentId, (incoming) =>
      setLines((current) => mergeLogLines(current, incoming, KEPT_LINES)),
    );
  }, [environmentId, paused]);

  const visible = visibleLogs(lines, search, SHOWN_LINES);

  return (
    <div className="flex h-full flex-col">
      <LogToolbar
        search={search}
        services={services}
        count={visible.length}
        paused={paused}
        onTogglePause={() => setPaused((current) => !current)}
      />
      <LogHistogram lines={visible} />
      <div className="mt-2 border-t" />
      <LogList lines={visible} serviceNames={serviceNames(services)} />
    </div>
  );
}
