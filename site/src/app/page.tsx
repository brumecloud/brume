import { SiteFooter } from "~/components/site-footer";
import { SiteHeader } from "~/components/site-header";
import { ClosingCta } from "~/components/sections/closing-cta";
import { Features } from "~/components/sections/features";
import { Hero } from "~/components/sections/hero";
import { HowItWorks } from "~/components/sections/how-it-works";
import { MistBand } from "~/components/sections/mist-band";
import { Problem } from "~/components/sections/problem";
import { Roadmap } from "~/components/sections/roadmap";
import { Security } from "~/components/sections/security";
import { StackMarquee } from "~/components/sections/stack-marquee";
import { site } from "~/content";

const structuredData = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: site.name,
  url: site.url,
  description: site.description,
  applicationCategory: "DeveloperApplication",
  operatingSystem: "Linux",
  isAccessibleForFree: true,
  offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
  sameAs: [site.repo],
};

export default function HomePage() {
  return (
    <>
      <script
        type="application/ld+json"
        // oxlint-disable-next-line react/no-danger -- static JSON-LD, with "<" escaped so it cannot close the script
        dangerouslySetInnerHTML={{
          __html: JSON.stringify(structuredData).replaceAll("<", "\\u003c"),
        }}
      />
      <SiteHeader />
      <main id="main">
        <Hero />
        <StackMarquee />
        <Problem />
        <HowItWorks />
        <Features />
        <MistBand />
        <Security />
        <Roadmap />
        <ClosingCta />
      </main>
      <SiteFooter />
    </>
  );
}
