import type { CSSProperties, ReactNode } from "react";

interface PinnedGridProps {
  // How many blocks the grid lights in turn.
  readonly count: number;
  readonly className?: string;
  readonly gridClassName: string;
  readonly children: ReactNode;
}

/**
 * A grid that stays still in the middle of the screen while the scroll lights its blocks one by
 * one, then moves on with the page. The spacer after it is the scroll the grid holds for. Only
 * where the whole grid fits on screen (globals.css); elsewhere it is a plain grid.
 */
export function PinnedGrid({ count, className = "", gridClassName, children }: PinnedGridProps) {
  return (
    <div data-pin className={className} style={{ "--blocks": count } as CSSProperties}>
      <div className={`pin-grid ${gridClassName}`}>{children}</div>
      <div aria-hidden="true" className="pin-spacer" />
    </div>
  );
}
