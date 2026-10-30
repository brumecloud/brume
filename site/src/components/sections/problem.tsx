import { CheckList } from "~/components/check-list";
import { Section } from "~/components/section";
import { usualSetup, withBrume } from "~/content";

export function Problem() {
  return (
    <Section
      id="problem"
      kicker="Why Brume"
      tone="rose"
      title="Self-hosting, secure by design."
      lede="Most self-hosted platforms run their admin panel on a public port, in the same process as the Docker socket. One bug in the panel, and an attacker is root on your machine."
    >
      <div className="mt-12 grid gap-7 md:grid-cols-2">
        <CheckList label="The usual setup" items={usualSetup} tone="no" />
        <CheckList label="With Brume" items={withBrume} tone="yes" strong />
      </div>
    </Section>
  );
}
