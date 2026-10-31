import { Card } from "~/components/card";
import { PinnedGrid } from "~/components/pinned-grid";
import { Section } from "~/components/section";
import { copy, features } from "~/content";

export function Features() {
  return (
    <Section id="features" title={copy.features.title}>
      <PinnedGrid
        count={features.length}
        className="mt-12"
        gridClassName="grid gap-6 sm:grid-cols-2 lg:grid-cols-4"
      >
        {features.map((feature) => (
          <Card key={feature.title} {...feature} />
        ))}
      </PinnedGrid>
    </Section>
  );
}
