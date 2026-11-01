import type { Metadata } from "next";

import { PageHero } from "~/components/page-hero";
import { ReadingLight } from "~/components/reading-light";
import { Section } from "~/components/section";
import { ClosingCta } from "~/components/sections/closing-cta";
import { SiteFooter } from "~/components/site-footer";
import { SiteHeader } from "~/components/site-header";
import { CompetitorCard } from "~/components/vs/competitor-card";
import { competitorsOfKind, vsCopy, vsGroups } from "~/components/vs/shared";
import { pageAt } from "~/content";
import { JsonLd, pageJsonLd } from "~/lib/json-ld";
import { pageMetadata } from "~/lib/page-metadata";

const page = pageAt("/vs");

export const metadata: Metadata = pageMetadata({
  path: page.path,
  title: { absolute: "Brume vs Coolify, Dokploy, Railway, Vercel and more" },
  description: page.description,
});

export default function VsIndexPage() {
  return (
    <>
      <JsonLd
        data={pageJsonLd({ path: page.path, name: page.title, description: page.description })}
      />
      <SiteHeader />
      <main id="main">
        <PageHero badge={vsCopy.badge} title={vsCopy.index.title} lede={vsCopy.index.lede} />
        {vsGroups.map((group) => (
          <Section key={group.kind} id={group.kind} title={group.title} className="pt-0">
            <ul className="mt-10 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
              {competitorsOfKind(group.kind).map((competitor) => (
                <li key={competitor.slug}>
                  <CompetitorCard competitor={competitor} />
                </li>
              ))}
            </ul>
          </Section>
        ))}
        <ClosingCta />
      </main>
      <SiteFooter />
      <ReadingLight />
    </>
  );
}
