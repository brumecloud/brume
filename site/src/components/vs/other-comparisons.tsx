import Link from "next/link";

import { BrandIcon } from "~/components/brand-icon";
import { logoLit } from "~/components/tone";
import type { Competitor } from "~/vs/model";

import { otherComparisons, vsCopy } from "./shared";

/** Links to the comparisons of the same family, at the bottom of a comparison page. */
export function OtherComparisons({ current }: { readonly current: Competitor }) {
  return (
    <nav aria-labelledby="others-title" className="mx-auto w-full max-w-page px-6 py-16">
      <h2 id="others-title" className="text-label text-ink-dim">
        {vsCopy.others}
      </h2>
      <ul className="mt-5 flex flex-wrap gap-3">
        {otherComparisons(current).map((other) => (
          <li key={other.slug}>
            <Link
              href={`/vs/${other.slug}`}
              className="group pixel-border flex items-center gap-2.5 bg-surface px-4 py-2.5 text-sm text-ink-muted transition-colors hover:bg-highlight hover:text-ink"
            >
              {other.icon ? (
                <BrandIcon name={other.icon} className={`size-4 shrink-0 ${logoLit}`} />
              ) : null}
              {vsCopy.title(other.name)}
            </Link>
          </li>
        ))}
      </ul>
    </nav>
  );
}
