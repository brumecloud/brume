import { Fragment, type CSSProperties, type ReactNode } from "react";

import { PlayOnView } from "./play-on-view";

interface TerminalProps {
  readonly title: string;
  /** One entry per line. */
  readonly lines: readonly ReactNode[];
  /** Wraps long lines instead of scrolling. */
  readonly wrap?: boolean;
}

/** A terminal window whose lines appear one by one when it scrolls into view. */
export function Terminal({ title, lines, wrap = false }: TerminalProps) {
  return (
    <PlayOnView className="pixel-border min-w-0 bg-terminal">
      <div className="flex items-center gap-1.5 border-b-2 border-edge px-3.5 py-2.5 font-mono text-[11.5px] text-ink-dim">
        <span aria-hidden="true" className="size-2 bg-rose-400/70" />
        <span aria-hidden="true" className="size-2 bg-amber-400/70" />
        <span aria-hidden="true" className="size-2 bg-emerald-400/70" />
        <span className="ml-2.5">{title}</span>
      </div>
      <pre
        className={`overflow-x-auto px-[18px] pt-4 pb-[18px] font-mono text-[12.5px] leading-[1.75] text-ink ${wrap ? "whitespace-pre-wrap [overflow-wrap:anywhere]" : ""}`}
      >
        {lines.map((line, index) => (
          // oxlint-disable-next-line react/no-array-index-key -- static lines, never reordered
          <Fragment key={index}>
            <span
              className="terminal-line inline-block"
              style={{ "--line": index } as CSSProperties}
            >
              {line}
              {index === lines.length - 1 ? (
                <span
                  aria-hidden="true"
                  className="ml-1 inline-block h-[1.1em] w-[0.55em] translate-y-[0.2em] animate-blink bg-ink-muted motion-reduce:animate-none"
                />
              ) : null}
            </span>
            {"\n"}
          </Fragment>
        ))}
      </pre>
    </PlayOnView>
  );
}

export function Prompt() {
  return <span className="text-amber-300">$ </span>;
}

export function Ok() {
  return <span className="text-emerald-400">✓ </span>;
}

export function Arrow() {
  return <span className="text-sky-400">→ </span>;
}

export function Dim({ children }: { readonly children: ReactNode }) {
  return <span className="text-ink-dim">{children}</span>;
}

export function Link({ children }: { readonly children: ReactNode }) {
  return (
    <span className="text-sky-300 underline decoration-sky-300/40 underline-offset-2">
      {children}
    </span>
  );
}
