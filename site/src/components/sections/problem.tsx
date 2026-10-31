import { CheckList } from "~/components/check-list";
import { Section } from "~/components/section";
import { copy, usualSetup, withBrume } from "~/content";

export function Problem() {
  return (
    <Section id="problem" title={copy.problem.title} lede={copy.problem.lede}>
      <div className="mt-12 grid gap-7 md:grid-cols-2">
        <CheckList label="The usual setup" items={usualSetup} tone="no" />
        <CheckList label="With Brume" items={withBrume} tone="yes" strong />
      </div>
    </Section>
  );
}
