import Image from "next/image";
import type { ReactNode } from "react";

import logo from "~/assets/logo.png";
import { BrandIcon } from "~/components/brand-icon";
import { logoLit } from "~/components/tone";
import type { Competitor } from "~/vs/model";

import { vsCopy } from "./shared";

/** "Pick <name> if…" and "Pick Brume if…": one line each. */
export function Picks({ competitor }: { readonly competitor: Competitor }) {
  return (
    <div className="grid gap-6 md:grid-cols-2">
      <Pick
        id="pick-them"
        title={vsCopy.pickThem(competitor.name)}
        text={competitor.pickThem}
        mark={
          competitor.icon ? (
            <BrandIcon name={competitor.icon} className={`size-5 shrink-0 ${logoLit}`} />
          ) : null
        }
      />
      <Pick
        id="pick-brume"
        title={vsCopy.pickBrume}
        text={competitor.pickBrume}
        mark={
          <Image
            src={logo}
            alt=""
            width={20}
            height={20}
            className={`shrink-0 rounded ${logoLit}`}
          />
        }
      />
    </div>
  );
}

interface PickProps {
  readonly id: string;
  readonly title: string;
  readonly text: string;
  readonly mark: ReactNode;
}

function Pick({ id, title, text, mark }: PickProps) {
  return (
    <section
      aria-labelledby={id}
      data-light
      className="group pixel-border reveal bg-surface p-6 transition-colors duration-300 hover:bg-highlight hover:[--pixel-border-color:var(--color-edge-strong)] data-lit:bg-highlight data-lit:[--pixel-border-color:var(--color-edge-strong)]"
    >
      <div className="flex items-center gap-2.5">
        {mark}
        <h2 id={id} className="text-label text-ink-dim transition-colors duration-300 lit:text-ink">
          {title}
        </h2>
      </div>
      <p className="mt-4 text-lg leading-snug font-medium tracking-[-0.01em] text-pretty">{text}</p>
    </section>
  );
}
