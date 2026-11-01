import type { Metadata } from "next";

import { architectureCopy } from "~/architecture/copy";
import { ComponentList } from "~/components/architecture/component-list";
import { Explorer } from "~/components/architecture/explorer";
import { PageHero } from "~/components/page-hero";
import { ReadingLight } from "~/components/reading-light";
import { Section } from "~/components/section";
import { ClosingCta } from "~/components/sections/closing-cta";
import { SiteFooter } from "~/components/site-footer";
import { SiteHeader } from "~/components/site-header";
import { pageAt } from "~/content";
import { JsonLd, pageJsonLd } from "~/lib/json-ld";
import { pageMetadata } from "~/lib/page-metadata";

const page = pageAt("/architecture");

export const metadata: Metadata = pageMetadata(page);

export default function ArchitecturePage() {
  return (
    <>
      <JsonLd
        data={pageJsonLd({
          path: page.path,
          name: architectureCopy.title,
          description: page.description,
        })}
      />
      <SiteHeader />
      <main id="main">
        <PageHero
          badge={architectureCopy.badge}
          title={architectureCopy.title}
          lede={architectureCopy.lede}
        />
        <section aria-labelledby="explorer-title" className="pb-12">
          <h2 id="explorer-title" className="sr-only">
            {architectureCopy.explorerTitle}
          </h2>
          <Explorer />
        </section>
        <Section id="principles" title={architectureCopy.principlesTitle}>
          <ol className="mt-10 grid gap-x-10 gap-y-6 sm:grid-cols-2 lg:grid-cols-3">
            {architectureCopy.principles.map((principle, index) => (
              <li
                key={principle}
                data-light
                className="group reveal flex gap-4 text-lg leading-snug font-medium tracking-[-0.01em]"
              >
                <span
                  aria-hidden="true"
                  className="text-label pt-1.5 text-ink-dim transition-colors duration-300 lit:text-brand"
                >
                  {String(index + 1).padStart(2, "0")}
                </span>
                {principle}
              </li>
            ))}
          </ol>
        </Section>
        <ComponentList />
        <ClosingCta />
      </main>
      <SiteFooter />
      <ReadingLight />
    </>
  );
}
