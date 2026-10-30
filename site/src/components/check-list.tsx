type Tone = "no" | "yes" | "later" | "need";

const markers: Record<Tone, { readonly mark: string; readonly className: string }> = {
  no: { mark: "×", className: "text-ink-dim" },
  yes: { mark: "✓", className: "text-brand" },
  later: { mark: "○", className: "text-ink-dim" },
  need: { mark: "›", className: "text-brand" },
};

interface CheckListProps {
  readonly label: string;
  readonly items: readonly string[];
  readonly tone: Tone;
  readonly strong?: boolean;
}

/** A labeled list in a pixel box. The label says what the marks mean. */
export function CheckList({ label, items, tone, strong = false }: CheckListProps) {
  const { mark, className } = markers[tone];
  const muted = tone === "no" || tone === "later";
  return (
    <div className={`pixel-border bg-surface p-7 ${strong ? "pixel-border-strong" : ""}`}>
      <p className="text-pixel text-ink-dim">{label}</p>
      <ul className="mt-5 grid gap-3">
        {items.map((item) => (
          <li key={item} className={`flex gap-3 leading-snug ${muted ? "text-ink-muted" : ""}`}>
            <span aria-hidden="true" className={`w-3 shrink-0 font-mono ${className}`}>
              {mark}
            </span>
            <span>{item}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
