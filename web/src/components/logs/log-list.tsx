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

interface LogListProps {
  readonly lines: readonly LogLine[];
  readonly serviceNames: ReadonlyMap<string, string>;
}

// Follows the stream while the admin stays at the bottom; scrolling up pauses the follow.
export function LogList({ lines, serviceNames }: LogListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [following, setFollowing] = useState(true);
  const newestLineId = lines.at(-1)?.id;

  // oxlint-disable-next-line brume/no-unreasoned-effect -- keeps the newest line in view while the admin follows the stream
  useEffect(() => {
    const element = scrollRef.current;
    if (following && element && newestLineId) {
      element.scrollTop = element.scrollHeight;
    }
  }, [following, newestLineId]);

  return (
    <div className="relative min-h-0 flex-1">
      <div
        ref={scrollRef}
        className="h-full overflow-auto py-2"
        onScroll={(event) => {
          const { scrollTop, clientHeight, scrollHeight } = event.currentTarget;
          setFollowing(isNearBottom(scrollTop, clientHeight, scrollHeight));
        }}
      >
        {lines.length === 0 ? (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <ScrollTextIcon />
              </EmptyMedia>
              <EmptyTitle>Aucune ligne</EmptyTitle>
              <EmptyDescription>
                Aucun log ne correspond à ces filtres pour l'instant.
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          lines.map((line) => (
            <LogRow
              key={line.id}
              line={line}
              serviceName={serviceNames.get(line.serviceId) ?? ""}
            />
          ))
        )}
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
