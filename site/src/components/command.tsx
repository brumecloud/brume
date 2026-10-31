import type { ReactNode } from "react";

interface CommandProps {
  readonly children: ReactNode;
  readonly className?: string;
}

/** A command to type, set apart from the prose: mono, on the terminal background, after a prompt. */
export function Command({ children, className = "" }: CommandProps) {
  return (
    <code
      className={`pixel-border inline-flex items-center gap-[0.5em] bg-terminal px-[0.6em] py-[0.3em] font-mono font-normal tracking-normal text-ink ${className}`}
    >
      <span
        aria-hidden="true"
        className="text-ink-dim transition-colors duration-300 lit:text-amber-300"
      >
        $
      </span>
      {children}
    </code>
  );
}
