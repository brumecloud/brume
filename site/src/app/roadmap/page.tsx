import type { Metadata } from "next";

import { PageHero } from "~/components/page-hero";
import { ReadingLight } from "~/components/reading-light";
import { RoadmapTimeline, milestoneStyles } from "~/components/roadmap-timeline";
import { Section } from "~/components/section";
import { ClosingCta } from "~/components/sections/closing-cta";
import { SiteFooter } from "~/components/site-footer";
import { SiteHeader } from "~/components/site-header";
import { TextLink } from "~/components/text-link";
import { docsPage } from "~/content";
import { pageAt } from "~/content";
import { JsonLd, pageJsonLd } from "~/lib/json-ld";
import { pageMetadata } from "~/lib/page-metadata";
import { openDecisions, roadmapCopy, roadmapThemes, themeId, whenLabel } from "~/roadmap";

const page = pageAt("/roadmap");

export const metadata: Metadata = pageMetadata(page);

export default function RoadmapPage() {
  return (
    <>
      <JsonLd
        data={pageJsonLd({
          path: page.path,
          name: roadmapCopy.title,
          description: page.description,
        })}
      />
      <SiteHeader />
      <main id="main">
        <PageHero badge={roadmapCopy.badge} title={roadmapCopy.title} lede={roadmapCopy.lede} />
        <section aria-labelledby="timeline-title" className="pb-12">
          <div className="mx-auto w-full max-w-page px-6">
            <h2 id="timeline-title" className="sr-only">
              {roadmapCopy.timelineTitle}
            </h2>
            <RoadmapTimeline />
          </div>
        </section>

        <Section id="planned" title={roadmapCopy.themesTitle} lede={roadmapCopy.themesLede}>
          <ul className="mt-12 grid gap-6">
            {roadmapThemes.map((theme) => (
              <li
                key={theme.title}
                id={themeId(theme.title)}
                data-light
                className="group pixel-border reveal grid scroll-mt-8 gap-6 bg-surface p-6 transition-colors duration-300 hover:bg-highlight hover:[--pixel-border-color:var(--color-edge-strong)] data-lit:bg-highlight data-lit:[--pixel-border-color:var(--color-edge-strong)] md:grid-cols-[11rem_minmax(0,1fr)] md:gap-8 md:p-8"
              >
                <h3 className="text-xl leading-tight font-semibold tracking-[-0.02em]">
                  {theme.title}
                </h3>
                <ul className="grid gap-x-10 gap-y-6 sm:grid-cols-2">
                  {theme.items.map((item) => (
                    <li key={item.title} className="flex gap-3">
                      <span
                        aria-hidden="true"
                        className={`mt-[0.55em] size-1.5 shrink-0 transition-colors duration-300 ${item.when ? milestoneStyles[item.when].marker : "bg-ink-dim"}`}
                      />
                      <div>
                        <div className="flex flex-wrap items-baseline gap-x-2.5 gap-y-1">
                          <h4 className="leading-snug font-semibold tracking-[-0.01em]">
                            {item.title}
                          </h4>
                          {item.when ? (
                            <span
                              className={`text-label transition-colors duration-300 ${milestoneStyles[item.when].label}`}
                            >
                              {whenLabel(item.when)}
                            </span>
                          ) : null}
                        </div>
                        <p className="mt-1.5 text-[15px] leading-relaxed text-pretty text-ink-muted">
                          {item.brings}
                        </p>
                      </div>
                    </li>
                  ))}
                </ul>
              </li>
            ))}
          </ul>
        </Section>

        <Section id="open" title={roadmapCopy.openTitle} lede={roadmapCopy.openLede}>
          <ul className="mt-10 border-t-2 border-edge">
            {openDecisions.map((decision) => (
              <li
                key={decision.title}
                data-light
                className="group reveal grid gap-1 border-b-2 border-edge py-4 md:grid-cols-[18rem_minmax(0,1fr)] md:gap-8"
              >
                <p className="flex gap-3 font-semibold tracking-[-0.01em]">
                  <span
                    aria-hidden="true"
                    className="w-3 shrink-0 text-ink-dim transition-colors duration-300 lit:text-brand"
                  >
                    ?
                  </span>
                  {decision.title}
                </p>
                <p className="pl-6 text-[15px] leading-relaxed text-pretty text-ink-muted md:pl-0">
                  {decision.text}
                </p>
              </li>
            ))}
          </ul>
          <p className="reveal mt-10 text-[17px] text-ink-muted">
            <TextLink href={docsPage("how-it-works/roadmap")}>{roadmapCopy.docsLink} ↗</TextLink>
          </p>
        </Section>
        <ClosingCta />
      </main>
      <SiteFooter />
      <ReadingLight />
    </>
  );
}
