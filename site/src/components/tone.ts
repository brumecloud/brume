// Accent colors, so the page is not blue everywhere. At rest the page stays in
// grays: an accent only shows while its block is lit, by the pointer or by the
// reading line (the `lit` variant in globals.css). Class names are written in
// full so Tailwind finds them.

export type Tone = "sky" | "emerald" | "amber" | "violet" | "rose" | "teal" | "orange" | "fuchsia";

export const toneTextLit: Record<Tone, string> = {
  sky: "lit:text-sky-400",
  emerald: "lit:text-emerald-400",
  amber: "lit:text-amber-400",
  violet: "lit:text-violet-400",
  rose: "lit:text-rose-400",
  teal: "lit:text-teal-400",
  orange: "lit:text-orange-400",
  fuchsia: "lit:text-fuchsia-400",
};

export const toneTileLit: Record<Tone, string> = {
  sky: "lit:bg-sky-400/10 lit:text-sky-300 lit:ring-sky-400/30",
  emerald: "lit:bg-emerald-400/10 lit:text-emerald-300 lit:ring-emerald-400/30",
  amber: "lit:bg-amber-400/10 lit:text-amber-300 lit:ring-amber-400/30",
  violet: "lit:bg-violet-400/10 lit:text-violet-300 lit:ring-violet-400/30",
  rose: "lit:bg-rose-400/10 lit:text-rose-300 lit:ring-rose-400/30",
  teal: "lit:bg-teal-400/10 lit:text-teal-300 lit:ring-teal-400/30",
  orange: "lit:bg-orange-400/10 lit:text-orange-300 lit:ring-orange-400/30",
  fuchsia: "lit:bg-fuchsia-400/10 lit:text-fuchsia-300 lit:ring-fuchsia-400/30",
};

/** Brand logos in gray, in color while their block is lit. */
export const logoLit =
  "grayscale opacity-75 transition duration-300 lit:grayscale-0 lit:opacity-100";
