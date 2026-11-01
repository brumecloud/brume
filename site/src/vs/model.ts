// The comparison pages, one per competitor: /vs/<slug>. Every fact about a competitor comes from
// its official docs, linked in `source`, as checked on `checked`. The research notes are in
// archi/jalons/2026-11-01-recherche-vs/.

import type { BrandIconName } from "~/components/brand-icons";

export type VsSlug =
  | "coolify"
  | "dokploy"
  | "caprover"
  | "portainer"
  | "kamal"
  | "dokku"
  | "easypanel"
  | "railway"
  | "render"
  | "fly"
  | "vercel"
  | "heroku";

/** A line of the comparison table. Its label and Brume's value live in brume.ts. */
export type RowKey =
  | "runs"
  | "panel"
  | "docker"
  | "hacked"
  | "ports"
  | "install"
  | "data"
  | "pricing"
  | "lockin"
  | "databases"
  | "backups"
  | "agents"
  | "servers"
  | "scaling"
  | "compliance"
  | "license"
  | "maturity";

export interface VsRow {
  readonly key: RowKey;
  /** The competitor's side, in a few words. */
  readonly them: string;
  /** The official page that says so. Required for any fact; omit only for "not offered". */
  readonly source?: string;
}

export interface Competitor {
  readonly slug: VsSlug;
  /** As the competitor writes it: "Fly.io". */
  readonly name: string;
  readonly kind: "self-hosted" | "managed";
  readonly icon?: BrandIconName;
  /** The page's <title>, without " · Brume". Under 60 characters, unique. */
  readonly seoTitle: string;
  /** The meta description. Under 160 characters, unique. */
  readonly description: string;
  /** The line under its name on /vs. */
  readonly oneLiner: string;
  /** One or two short sentences, fair to both. */
  readonly verdict: string;
  /** About eight lines, in the order of `selfHostedRows` or `managedRows` in brume.ts. */
  readonly rows: readonly VsRow[];
  /** "Where Brume is different": three lines, under 15 words each. */
  readonly different: readonly string[];
  /** "Where <name> is better today": three lines, under 15 words each. */
  readonly better: readonly string[];
  /** "Pick <name> if…": one line. */
  readonly pickThem: string;
  /** "Pick Brume if…": one line. */
  readonly pickBrume: string;
  /** The day the facts were checked, ISO: "2026-11-01". */
  readonly checked: string;
}
