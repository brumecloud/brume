// Accent colors, so the page is not blue everywhere. Class names are written in
// full so Tailwind finds them.

export type Tone = "sky" | "emerald" | "amber" | "violet" | "rose" | "teal" | "orange" | "fuchsia";

export const toneText: Record<Tone, string> = {
  sky: "text-sky-400",
  emerald: "text-emerald-400",
  amber: "text-amber-400",
  violet: "text-violet-400",
  rose: "text-rose-400",
  teal: "text-teal-400",
  orange: "text-orange-400",
  fuchsia: "text-fuchsia-400",
};

export const toneTile: Record<Tone, string> = {
  sky: "bg-sky-400/10 text-sky-300 ring-sky-400/30",
  emerald: "bg-emerald-400/10 text-emerald-300 ring-emerald-400/30",
  amber: "bg-amber-400/10 text-amber-300 ring-amber-400/30",
  violet: "bg-violet-400/10 text-violet-300 ring-violet-400/30",
  rose: "bg-rose-400/10 text-rose-300 ring-rose-400/30",
  teal: "bg-teal-400/10 text-teal-300 ring-teal-400/30",
  orange: "bg-orange-400/10 text-orange-300 ring-orange-400/30",
  fuchsia: "bg-fuchsia-400/10 text-fuchsia-300 ring-fuchsia-400/30",
};
