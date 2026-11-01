// What the comparison pages and their Markdown twins share: the few words around the data in
// src/vs/, and the lookups. Every fact lives in src/vs/, not here.

import { competitors } from "~/vs";
import type { Competitor } from "~/vs/model";

export const vsCopy = {
  badge: "Alpha 0.1 · in progress",
  title: (name: string) => `Brume vs ${name}`,
  checked: (date: string) =>
    `Facts checked on ${date} in each project's official docs. "If the panel is hacked" is our reading of them. Brume is shown in its recommended modes: Cloudflare Tunnel and Tailscale.`,
  different: "Where Brume is different",
  better: (name: string) => `Where ${name} is better today`,
  pickThem: (name: string) => `Pick ${name} if…`,
  pickBrume: "Pick Brume if…",
  others: "Other comparisons",
  index: {
    title: "Compare Brume.",
    lede: "Fact by fact, with sources, and where the others do better.",
  },
} as const;

/** The two families of /vs, in page order. */
export const vsGroups: readonly { readonly kind: Competitor["kind"]; readonly title: string }[] = [
  { kind: "self-hosted", title: "Self-hosted platforms" },
  { kind: "managed", title: "Managed platforms" },
];

/** "2026-11-01" as "November 1, 2026". */
export function checkedOn(iso: string): string {
  return new Date(`${iso}T00:00:00Z`).toLocaleDateString("en-US", {
    dateStyle: "long",
    timeZone: "UTC",
  });
}

/** The comparison at /vs/<slug>, for a slug read from the URL. */
export function findCompetitor(slug: string): Competitor | undefined {
  return competitors.find((candidate) => candidate.slug === slug);
}

export function competitorsOfKind(kind: Competitor["kind"]): readonly Competitor[] {
  return competitors.filter((candidate) => candidate.kind === kind);
}

/** The other comparisons of the same family, linked at the bottom of a page. */
export function otherComparisons(current: Competitor): readonly Competitor[] {
  return competitorsOfKind(current.kind).filter((other) => other.slug !== current.slug);
}
