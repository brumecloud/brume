import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { PageHero } from "~/components/page-hero";
import { ReadingLight } from "~/components/reading-light";
import { ClosingCta } from "~/components/sections/closing-cta";
import { SiteFooter } from "~/components/site-footer";
import { SiteHeader } from "~/components/site-header";
import { ComparisonTable } from "~/components/vs/comparison-table";
import { OtherComparisons } from "~/components/vs/other-comparisons";
import { Picks } from "~/components/vs/picks";
import { vsCopy } from "~/components/vs/shared";
import { Strengths } from "~/components/vs/strengths";
import { JsonLd, pageJsonLd } from "~/lib/json-ld";
import { pageMetadata } from "~/lib/page-metadata";
import { competitors, findCompetitor } from "~/vs";
import type { Competitor } from "~/vs/model";

interface VsPageProps {
  readonly params: Promise<{ slug: string }>;
}

// One page per comparison in src/vs/, and no other.
export const dynamicParams = false;

export function generateStaticParams(): { slug: string }[] {
  return competitors.map(({ slug }) => ({ slug }));
}

async function competitorOf(params: VsPageProps["params"]): Promise<Competitor> {
  const found = findCompetitor((await params).slug);
  if (!found) notFound();
  return found;
}

export async function generateMetadata({ params }: VsPageProps): Promise<Metadata> {
  const competitor = await competitorOf(params);
  return pageMetadata({
    path: `/vs/${competitor.slug}`,
    title: { absolute: competitor.seoTitle },
    description: competitor.description,
  });
}

export default async function VsPage({ params }: VsPageProps) {
  const competitor = await competitorOf(params);
  return (
    <>
      <JsonLd
        data={pageJsonLd({
          path: `/vs/${competitor.slug}`,
          name: competitor.seoTitle,
          description: competitor.description,
          parents: [{ name: "Compare", path: "/vs" }],
          extra: { mentions: { "@type": "SoftwareApplication", name: competitor.name } },
        })}
      />
      <SiteHeader />
      <main id="main">
        <PageHero
          badge={vsCopy.badge}
          title={vsCopy.title(competitor.name)}
          lede={competitor.verdict}
        />
        <div className="mx-auto w-full max-w-page px-6">
          <ComparisonTable competitor={competitor} />
        </div>
        <div className="mx-auto grid w-full max-w-page gap-20 px-6 py-24">
          <Strengths competitor={competitor} />
          <Picks competitor={competitor} />
        </div>
        <ClosingCta />
        <OtherComparisons current={competitor} />
      </main>
      <SiteFooter />
      <ReadingLight />
    </>
  );
}
