import Link from "next/link";

import { RoadmapTimeline } from "~/components/roadmap-timeline";
import { Section } from "~/components/section";
import { copy } from "~/content";

export function Roadmap() {
  return (
    <Section id="roadmap" title={copy.roadmap.title}>
      <RoadmapTimeline className="mt-14" />
      <p className="reveal mt-12 text-[17px] text-ink-muted">
        <Link
          href="/roadmap"
          className="border-b border-dashed border-edge-strong pb-0.5 transition-colors hover:text-brand"
        >
          Full roadmap →
        </Link>
      </p>
    </Section>
  );
}
