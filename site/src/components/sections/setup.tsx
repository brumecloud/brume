import { Card } from "~/components/card";
import { Command } from "~/components/command";
import { PinnedGrid } from "~/components/pinned-grid";
import { Section } from "~/components/section";
import { Arrow, Link, Ok, Prompt, Terminal } from "~/components/terminal";
import { copy, setupFeatures, site, steps } from "~/content";

export function Setup() {
  return (
    <Section id="setup" title={copy.setup.title} lede={copy.setup.lede}>
      <div className="mt-14 grid items-center gap-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)] lg:gap-14">
        <ol className="grid gap-8">
          {steps.map((step, index) => (
            <li key={step.id} data-light className="group reveal flex gap-5">
              <span
                aria-hidden="true"
                className="text-label w-5 shrink-0 pt-1.5 text-ink-dim transition-colors duration-300 lit:text-brand"
              >
                {index + 1}
              </span>
              <div>
                <h3 className="text-xl font-semibold tracking-[-0.02em]">
                  {step.command ? <Command className="text-lg">{step.title}</Command> : step.title}
                </h3>
                <p className="mt-2 leading-relaxed text-ink-muted">{step.text}</p>
              </div>
            </li>
          ))}
        </ol>
        {/* The output is illustrative until the alpha ships. */}
        <div data-light className="reveal">
          <Terminal
            title="root@vps"
            wrap
            lines={[
              <>
                <Prompt />
                {site.installCommand}
              </>,
              <>
                <Ok />
                signature verified
              </>,
              <>
                <Ok />
                firewall closed, no inbound ports
              </>,
              <>
                <Ok />
                joined your tailnet
              </>,
              <>
                <Ok />
                cloudflare tunnel ready for *.example.com
              </>,
              <>
                <Ok />
                state saved to s3://brume-state
              </>,
              <>
                <Arrow />
                panel: <Link>https://brume.your-tailnet.ts.net</Link>
              </>,
            ]}
          />
        </div>
      </div>

      <PinnedGrid
        count={setupFeatures.length}
        className="mt-20"
        gridClassName="grid gap-6 sm:grid-cols-2 lg:grid-cols-4"
      >
        {setupFeatures.map((feature) => (
          <Card key={feature.title} {...feature} />
        ))}
      </PinnedGrid>
    </Section>
  );
}
