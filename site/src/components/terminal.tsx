import { Fragment, type ReactNode } from "react";

interface TerminalProps {
  readonly title: string;
  /** One entry per line. */
  readonly lines: readonly ReactNode[];
  /** Wraps long lines instead of scrolling. */
  readonly wrap?: boolean;
}

export function Terminal({ title, lines, wrap = false }: TerminalProps) {
  return (
    <div className="pixel-border min-w-0 bg-terminal">
      <div className="flex items-center gap-1.5 border-b-2 border-edge px-3.5 py-2.5 font-mono text-[11.5px] text-ink-dim">
        <span aria-hidden="true" className="size-2 bg-edge-strong" />
        <span aria-hidden="true" className="size-2 bg-edge-strong" />
        <span aria-hidden="true" className="size-2 bg-edge-strong" />
        <span className="ml-2.5">{title}</span>
      </div>
      <pre
        className={`overflow-x-auto px-[18px] pt-4 pb-[18px] font-mono text-[12.5px] leading-[1.75] text-ink ${wrap ? "whitespace-pre-wrap [overflow-wrap:anywhere]" : ""}`}
      >
        {lines.map((line, index) => (
          // oxlint-disable-next-line react/no-array-index-key -- static lines, never reordered
          <Fragment key={index}>
            {line}
            {"\n"}
          </Fragment>
        ))}
      </pre>
    </div>
  );
}

export function Prompt() {
  return <span className="text-brand">$ </span>;
}

export function Ok() {
  return <span className="text-brand">✓ </span>;
}

export function Arrow() {
  return <span className="text-brand">→ </span>;
}

export function Dim({ children }: { readonly children: ReactNode }) {
  return <span className="text-ink-dim">{children}</span>;
}

export function Link({ children }: { readonly children: ReactNode }) {
  return <span className="text-brand-soft">{children}</span>;
}
