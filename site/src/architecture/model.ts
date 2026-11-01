// The architecture page's model: the components of Brume, the links between them and the
// tours that walk through them. The content comes from docs/content/docs/how-it-works/.

import type { BrandIconName } from "~/components/brand-icons";

import { archEdges } from "./edges";

export type ComponentId =
  | "internet"
  | "cloudflare"
  | "traefik"
  | "caddy"
  | "apps"
  | "webhook"
  | "tailnet"
  | "api"
  | "engine"
  | "agent"
  | "buildkit"
  | "docker"
  | "telemetry"
  | "updater"
  | "s3";

/** The path a link belongs to. Each one has its own color while the diagram is lit. */
export type Flow = "public" | "admin" | "socket" | "control" | "storage" | "docker";

export type Glyph = "globe" | "storage" | "apps";

export type NodeIcon =
  | { readonly kind: "brand"; readonly name: BrandIconName }
  | { readonly kind: "glyph"; readonly name: Glyph }
  | { readonly kind: "brume" };

export interface SecurityRule {
  readonly rule: string;
  readonly why: string;
}

export interface ArchComponent {
  readonly id: ComponentId;
  readonly label: string;
  /** Under the label, in the diagram. */
  readonly detail: string;
  /** A Brume service, a third-party piece running on the host, or something outside it. */
  readonly kind: "brume" | "host" | "external";
  readonly icon: NodeIcon;
  /** The cell of the diagram's grid: see `COLUMNS` and `ROWS` in edges.ts. */
  readonly column: 0 | 1 | 2 | 3;
  readonly row: 0 | 1 | 2 | 3 | 4 | 5;
  /** One short sentence, for the panel beside the diagram. */
  readonly role: string;
  readonly why: string;
  readonly how: readonly string[];
  readonly rules: readonly SecurityRule[];
  /** What an attacker gets. Absent where the docs say nothing about it. */
  readonly ifCompromised?: string;
  /** The same, in a few words, for the panel. Present exactly when `ifCompromised` is. */
  readonly risk?: string;
  /** The docs page that owns the topic, for `docsPage()`. */
  readonly docs: string;
}

export interface ArchEdge {
  readonly id: `${ComponentId}-${ComponentId}`;
  readonly from: ComponentId;
  readonly to: ComponentId;
  readonly flow: Flow;
  readonly points: readonly (readonly [number, number])[];
  readonly label?: {
    readonly x: number;
    readonly y: number;
    readonly text: string;
    readonly anchor?: "start" | "middle" | "end";
  };
}

/** The components linked to this one, in either direction. */
export function neighbors(id: ComponentId): readonly ComponentId[] {
  const found = new Set<ComponentId>();
  for (const edge of archEdges) {
    if (edge.from === id) found.add(edge.to);
    if (edge.to === id) found.add(edge.from);
  }
  return [...found];
}

export function edgesOf(id: ComponentId): readonly ArchEdge[] {
  return archEdges.filter((edge) => edge.from === id || edge.to === id);
}
