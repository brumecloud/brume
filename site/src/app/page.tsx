import { ReadingLight } from "~/components/reading-light";
import { SiteFooter } from "~/components/site-footer";
import { SiteHeader } from "~/components/site-header";
import { ClosingCta } from "~/components/sections/closing-cta";
import { ConsoleDemo } from "~/components/sections/console-demo";
import { Deploy } from "~/components/sections/deploy";
import { Hero } from "~/components/sections/hero";
import { MistBand } from "~/components/sections/mist-band";
import { Roadmap } from "~/components/sections/roadmap";
import { Security } from "~/components/sections/security";
import { Setup } from "~/components/sections/setup";
import { JsonLd, brumeSoftware } from "~/lib/json-ld";

export default function HomePage() {
  return (
    <>
      <JsonLd data={{ "@context": "https://schema.org", ...brumeSoftware }} />
      <SiteHeader />
      <main id="main">
        <Hero />
        <Deploy />
        <MistBand />
        <Security />
        <ConsoleDemo />
        <Setup />
        <Roadmap />
        <ClosingCta />
      </main>
      <SiteFooter />
      <ReadingLight />
    </>
  );
}
