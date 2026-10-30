type Mark = "no" | "yes" | "later" | "need";

const markers: Record<Mark, { readonly mark: string; readonly className: string }> = {
  no: { mark: "×", className: "text-rose-400" },
  yes: { mark: "✓", className: "text-emerald-400" },
  later: { mark: "○", className: "text-violet-400" },
  need: { mark: "›", className: "text-amber-400" },
};

interface CheckListProps {
  readonly label: string;
  readonly items: readonly string[];
  readonly tone: Mark;
  readonly strong?: boolean;
}

/** A labeled list in a pixel box. The label says what the marks mean. */
export function CheckList({ label, items, tone, strong = false }: CheckListProps) {
  const { mark, className } = markers[tone];
  const muted = tone === "no" || tone === "later";
  return (
    <div className={`pixel-border reveal bg-surface p-7 ${strong ? "pixel-border-strong" : ""}`}>
      <p className="text-label text-ink-dim">{label}</p>
      <ul className="mt-5 grid gap-3">
        {items.map((item) => (
          <li key={item} className={`flex gap-3 leading-snug ${muted ? "text-ink-muted" : ""}`}>
            <span aria-hidden="true" className={`w-3 shrink-0 ${className}`}>
              {mark}
            </span>
            <span>{item}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
