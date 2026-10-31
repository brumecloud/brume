import type { LogLine } from "~/api/types";
import { logHistogram } from "~/helpers/logs";

import { LEVEL_COLORS, STACKED_LEVELS } from "./log-levels";

const BUCKETS = 60;

// Volume of the visible lines over time. Each bar stacks its levels, sized by their count.
export function LogHistogram({ lines }: { readonly lines: readonly LogLine[] }) {
  return (
    <div aria-hidden="true" className="flex h-10 items-end gap-px px-4">
      {logHistogram(lines, BUCKETS).map((bar, index) => (
        <span
          // oxlint-disable-next-line react/no-array-index-key -- a bucket is its position in time
          key={index}
          className="flex min-h-0.5 flex-1 flex-col-reverse overflow-hidden rounded-xs bg-muted"
          style={{ height: `${bar.height}%` }}
        >
          {STACKED_LEVELS.map(
            (level) =>
              bar.counts[level] > 0 && (
                <span
                  key={level}
                  className={LEVEL_COLORS[level]}
                  style={{ flexGrow: bar.counts[level] }}
                />
              ),
          )}
        </span>
      ))}
    </div>
  );
}
