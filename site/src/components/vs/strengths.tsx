import type { Competitor } from "~/vs/model";

import { vsCopy } from "./shared";

/** "Where Brume is different" and "Where <name> is better today", side by side. */
export function Strengths({ competitor }: { readonly competitor: Competitor }) {
  return (
    <div className="grid gap-x-10 gap-y-14 md:grid-cols-2">
      <Points
        id="different"
        title={vsCopy.different}
        points={competitor.different}
        markerLit="lit:bg-brand"
      />
      <Points
        id="better"
        title={vsCopy.better(competitor.name)}
        points={competitor.better}
        markerLit="lit:bg-amber-400"
      />
    </div>
  );
}

interface PointsProps {
  readonly id: string;
  readonly title: string;
  readonly points: readonly string[];
  /** The bullets' color while the list is lit, written in full so Tailwind finds it. */
  readonly markerLit: string;
}

function Points({ id, title, points, markerLit }: PointsProps) {
  return (
    <section aria-labelledby={`${id}-title`} data-light className="group reveal">
      <h2
        id={`${id}-title`}
        className="text-[clamp(1.5rem,2.4vw,1.875rem)] leading-tight font-semibold tracking-[-0.03em] text-balance"
      >
        {title}
      </h2>
      <ul className="mt-6 grid gap-3.5 text-[17px] leading-relaxed text-pretty text-ink-muted">
        {points.map((point) => (
          <li key={point} className="flex gap-3">
            <span
              aria-hidden="true"
              className={`mt-[0.7em] size-1.5 shrink-0 bg-ink-dim transition-colors duration-300 ${markerLit}`}
            />
            {point}
          </li>
        ))}
      </ul>
    </section>
  );
}
