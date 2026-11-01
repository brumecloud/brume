import { Card } from "~/components/card";
import { PinnedGrid } from "~/components/pinned-grid";
import { SectionHeading } from "~/components/section";
import { Arrow, Link, Ok, Prompt, Terminal } from "~/components/terminal";
import { copy, deployFeatures } from "~/content";

import { StackMarquee } from "./stack-marquee";

export function Deploy() {
  return (
    <section id="deploy" aria-labelledby="deploy-title" className="scroll-mt-8 pt-24 pb-12">
      <div className="mx-auto grid w-full max-w-page items-center gap-10 px-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] lg:gap-14">
        <SectionHeading id="deploy-title" title={copy.deploy.title} lede={copy.deploy.lede} />
        {/* The output is illustrative until the alpha ships. */}
        <div data-light className="reveal">
          <Terminal
            title="~/acme/api"
            lines={[
              <>
                <Prompt />
                git push origin main
              </>,
              <>
                <Arrow />
                build #42 · railpack · node 22
              </>,
              <>
                <Ok />
                healthy, traffic switched
              </>,
              <>
                <Ok />
                live at <Link>https://api.example.com</Link>
              </>,
            ]}
          />
        </div>
      </div>
      {/* Wider than the text: the band takes the width of the screen. */}
      <StackMarquee />
      <div className="mx-auto w-full max-w-page px-6">
        <PinnedGrid
          count={deployFeatures.length}
          gridClassName="grid gap-6 sm:grid-cols-2 lg:grid-cols-4"
        >
          {deployFeatures.map((feature) => (
            <Card key={feature.title} {...feature} />
          ))}
        </PinnedGrid>
      </div>
    </section>
  );
}
