import type { LogLine } from "~/api/types";
import { barHeights, logHistogram } from "~/helpers/logs";

const BUCKETS = 60;

// Volume of the visible lines over time, like the bars of infra/screens/03f-project-logs.png.
export function LogHistogram({ lines }: { readonly lines: readonly LogLine[] }) {
  const heights = barHeights(logHistogram(lines, BUCKETS));

  return (
    <div aria-hidden="true" className="flex h-10 items-end gap-px px-4">
      {heights.map((height, index) => (
        <span
          // oxlint-disable-next-line react/no-array-index-key -- a bucket is its position in time
          key={index}
          className="min-h-0.5 flex-1 rounded-xs bg-info/70"
          style={{ height: `${height}%` }}
        />
      ))}
    </div>
  );
}
