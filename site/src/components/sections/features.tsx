import { Card } from "~/components/card";
import { Section } from "~/components/section";
import { features } from "~/content";

export function Features() {
  return (
    <Section
      id="features"
      kicker="Features"
      title="Everything your apps need to run. Nothing you have to babysit."
    >
      <div className="mt-12 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
        {features.map((feature) => (
          <Card key={feature.title} {...feature} />
        ))}
      </div>
    </Section>
  );
}
