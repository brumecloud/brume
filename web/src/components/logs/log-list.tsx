import { useVirtualizer } from "@tanstack/react-virtual";
import { ArrowDownIcon, ScrollTextIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import type { LogLine } from "~/api/types";
import { Button } from "~/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";
import { isNearBottom } from "~/helpers/logs";

import { LogRow } from "./log-row";

const ESTIMATED_ROW_HEIGHT = 28;

interface LogListProps {
  readonly lines: readonly LogLine[];
  readonly serviceNames: ReadonlyMap<string, string>;
}

// Only the rows in view are rendered, each measured since a line can wrap. The React Compiler
// leaves this component uncompiled on purpose (see build/react-compiler.test.ts): keep it to
// the virtualizer glue, the rows themselves are compiled.
export function LogList({ lines, serviceNames }: LogListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [following, setFollowing] = useState(true);
  // oxlint-disable-next-line react/incompatible-library -- the compiler skips this component on purpose, see the comment above
  const virtualizer = useVirtualizer({
    count: lines.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ESTIMATED_ROW_HEIGHT,
    getItemKey: (index) => lines[index]?.id ?? index,
    overscan: 20,
  });
  const newestLineId = lines.at(-1)?.id;

  // oxlint-disable-next-line brume/no-unreasoned-effect -- keeps the newest line in view while the admin follows the stream
  useEffect(() => {
    if (following && newestLineId) {
      virtualizer.scrollToIndex(lines.length - 1, { align: "end" });
    }
  }, [following, newestLineId, lines.length, virtualizer]);

  if (lines.length === 0) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <ScrollTextIcon />
          </EmptyMedia>
          <EmptyTitle>Aucune ligne</EmptyTitle>
          <EmptyDescription>Aucun log ne correspond à ces filtres pour l'instant.</EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  return (
    <div className="relative min-h-0 flex-1 bg-terminal">
      <div
        ref={scrollRef}
        className="h-full overflow-auto"
        onScroll={(event) => {
          const { scrollTop, clientHeight, scrollHeight } = event.currentTarget;
          setFollowing(isNearBottom(scrollTop, clientHeight, scrollHeight));
        }}
      >
        <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
          {virtualizer.getVirtualItems().map((item) => {
            const line = lines[item.index];
            return (
              line && (
                <div
                  key={item.key}
                  ref={virtualizer.measureElement}
                  data-index={item.index}
                  className="absolute top-0 left-0 w-full"
                  style={{ transform: `translateY(${item.start}px)` }}
                >
                  <LogRow line={line} serviceName={serviceNames.get(line.serviceId) ?? ""} />
                </div>
              )
            );
          })}
        </div>
      </div>
      {!following && (
        <Button
          className="absolute right-6 bottom-4"
          variant="outline"
          size="sm"
          onClick={() => setFollowing(true)}
        >
          <ArrowDownIcon />
          Suivre le flux
        </Button>
      )}
    </div>
  );
}
