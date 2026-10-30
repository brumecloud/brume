import type { Feature } from "~/content";

import { BrandIcon } from "./brand-icon";
import { brandIcons } from "./brand-icons";
import { HoverAnimate } from "./hover-animate";
import { IconTile } from "./icon-tile";
import { toneText } from "./tone";

/** A feature card. Hovering it plays its icon's animation. */
export function Card({ label, title, text, icon, tone, logos }: Feature) {
  return (
    <HoverAnimate
      as="article"
      className="pixel-border reveal bg-surface p-6 transition-colors duration-300 hover:bg-highlight hover:[--pixel-border-color:var(--color-edge-strong)]"
    >
      <div className="flex items-center justify-between gap-3">
        <IconTile icon={icon} tone={tone} />
        <p className={`text-label ${toneText[tone]}`}>{label}</p>
      </div>
      <h3 className="mt-5 text-[17px] leading-snug font-semibold tracking-[-0.015em]">{title}</h3>
      <p className="mt-2.5 text-[15px] leading-relaxed text-ink-muted">{text}</p>
      {logos ? (
        <ul className="mt-4 flex flex-wrap gap-3">
          {logos.map((logo) => (
            <li key={logo} className="flex items-center gap-1.5 text-xs text-ink-muted">
              <BrandIcon name={logo} className="size-4" />
              {brandIcons[logo].title}
            </li>
          ))}
        </ul>
      ) : null}
    </HoverAnimate>
  );
}
