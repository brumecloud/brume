import type { Log } from "@/schemas/log.schema";
import { liveLogs } from "@/state/live-log.state";
import { useEffect, useRef } from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";
import { useSnapshot } from "valtio";

import { LogLine } from "./logs-body.comp";

type LogsRenderProps = {
  logs: Log[];
  logsSubscription: () => void;
};

export const LogsRender = ({
  logsSubscription,
  logs,
}: LogsRenderProps) => {
  const virtuosoScrollerRef = useRef<VirtuosoHandle>(null);
  const { isLive, stopLive } = useSnapshot(liveLogs);

  // used to subscribe to the logs
  // and to scroll to the bottom when the sub is finished (because we are at the top by defautl)
  useEffect(() => {
    if (isLive) {
      const close = logsSubscription();
      return close;
    }
    // saying crap, if we add logSubcription we get an infinite loop
    // well play react linter
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isLive]);

  useEffect(() => {
    if (isLive) {
      virtuosoScrollerRef.current?.scrollToIndex({
        behavior: "auto",
        index: "LAST",
      });
    }
  }, [logs.length, isLive]);

  return (
    <div
      className="h-full max-h-full cursor-text overflow-x-auto font-mono"
      onWheel={(e) => {
        if (isLive && e.deltaY < 0) {
          stopLive();
        }
      }}
      onMouseDown={() => stopLive()}>
      <div className="h-full w-full">
        <Virtuoso
          ref={virtuosoScrollerRef}
          data={logs}
          totalCount={logs.length}
          itemContent={(_, data) => (
            <LogLine key={data.id} data={data} />
          )}
        />
      </div>
    </div>
  );
};
