import type { ComponentType } from "react";

import { type Tone, toneTile } from "./tone";

/** An Animate UI icon: it animates when a parent `AnimateIcon` is hovered. */
export type AnimatedIcon = ComponentType<{ readonly className?: string; readonly size?: number }>;

interface IconTileProps {
  readonly icon: AnimatedIcon;
  readonly tone: Tone;
}

export function IconTile({ icon: Icon, tone }: IconTileProps) {
  return (
    <span
      aria-hidden="true"
      className={`grid size-10 shrink-0 place-items-center rounded-md ring-1 ring-inset ${toneTile[tone]}`}
    >
      <Icon size={20} className="[stroke-width:1.75]" />
    </span>
  );
}
