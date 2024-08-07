import type { Log } from "@/schemas/log.schema";
import { useEffect, useRef } from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";

import { LogLine } from "./logs-body.comp";

type LogsRenderProps = {
  logs: Log[];
  isLive: boolean;
  logsSubscription: () => void;
};

export const LogsRender = ({
  logsSubscription,
  isLive,
  logs,
}: LogsRenderProps) => {
  const scroller = useRef<VirtuosoHandle>(null);
  useEffect(() => {
    const close = logsSubscription();
    return close;
  }, [isLive]);

  useEffect(() => {
    if (scroller.current) {
      scroller.current?.autoscrollToBottom();
    }
  }, [logs]);

  return (
    <div className="h-full max-h-full cursor-text overflow-x-auto font-mono">
      <div className="h-full w-full">
        <Virtuoso
          ref={scroller}
          style={{ height: "100%", flex: 1 }}
          data={logs}
          totalCount={logs.length}
          itemContent={(_, data) => <LogLine data={data} />}
          followOutput
          alignToBottom
        />
      </div>
    </div>
  );
};
