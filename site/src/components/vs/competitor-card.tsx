import Link from "next/link";

import { BrandIcon } from "~/components/brand-icon";
import { logoLit } from "~/components/tone";
import type { Competitor } from "~/vs/model";

import { vsCopy } from "./shared";

/** A comparison on /vs: the competitor's logo, the page's name and one line, linked to the page. */
export function CompetitorCard({ competitor }: { readonly competitor: Competitor }) {
  return (
    <Link
      href={`/vs/${competitor.slug}`}
      data-light
      className="group pixel-border reveal flex h-full flex-col bg-surface p-6 transition-colors duration-300 hover:bg-highlight hover:[--pixel-border-color:var(--color-edge-strong)] data-lit:bg-highlight data-lit:[--pixel-border-color:var(--color-edge-strong)]"
    >
      <span className="flex h-6 items-center justify-between gap-3">
        {competitor.icon ? (
          <BrandIcon name={competitor.icon} className={`size-6 ${logoLit}`} />
        ) : (
          <span />
        )}
        <span
          aria-hidden="true"
          className="text-ink-dim transition-colors duration-300 lit:text-brand"
        >
          →
        </span>
      </span>
      <h3 className="mt-5 text-[17px] leading-snug font-semibold tracking-[-0.015em]">
        {vsCopy.title(competitor.name)}
      </h3>
      <p className="mt-2.5 text-[15px] leading-relaxed text-pretty text-ink-muted">
        {competitor.oneLiner}
      </p>
    </Link>
  );
}
