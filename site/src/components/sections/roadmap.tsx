import { CheckList } from "~/components/check-list";
import { Section } from "~/components/section";
import { roadmap } from "~/content";

export function Roadmap() {
  return (
    <Section id="alpha" kicker="Alpha 0.1" title="What ships first, and what comes next.">
      <div className="mt-12 grid gap-7 lg:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)_minmax(0,1fr)]">
        <CheckList label="In 0.1" items={roadmap.shipping} tone="yes" strong />
        <CheckList label="Later" items={roadmap.later} tone="later" />
        <CheckList label="You'll need" items={roadmap.requirements} tone="need" />
      </div>
    </Section>
  );
}
