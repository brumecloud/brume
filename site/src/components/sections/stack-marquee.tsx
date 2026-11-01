import { BrandIcon } from "~/components/brand-icon";
import { logoLit } from "~/components/tone";
import { copy, stackItems } from "~/content";

function StackList({ hidden = false }: { readonly hidden?: boolean }) {
  return (
    <ul aria-hidden={hidden || undefined} className="flex shrink-0 items-center gap-10 pr-10">
      {stackItems.map((item) => (
        <li
          key={item.name}
          className="group flex items-center gap-2.5 text-[15px] whitespace-nowrap text-ink-muted"
        >
          <BrandIcon name={item.icon} className={`size-6 ${logoLit}`} />
          {item.name}
        </li>
      ))}
    </ul>
  );
}

/** A slow, endless band of the stacks people bring. Hovering pauses it. */
export function StackMarquee() {
  return (
    <div className="py-14">
      <h3 className="text-label text-center text-ink-dim">{copy.deploy.stack}</h3>
      <div
        data-light
        className="mt-7 overflow-hidden [mask-image:linear-gradient(to_right,transparent,black_12%,black_88%,transparent)]"
      >
        <div className="flex w-max animate-marquee hover:[animation-play-state:paused] motion-reduce:animate-none">
          <StackList />
          {/* The copy makes the loop seamless: the track moves by half its width. */}
          <StackList hidden />
        </div>
      </div>
    </div>
  );
}
