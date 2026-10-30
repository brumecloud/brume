import { BrandIcon } from "~/components/brand-icon";
import { stackItems } from "~/content";

function StackList({ hidden = false }: { readonly hidden?: boolean }) {
  return (
    <ul aria-hidden={hidden || undefined} className="flex shrink-0 items-center gap-10 pr-10">
      {stackItems.map((item) => (
        <li
          key={item.name}
          className="flex items-center gap-2.5 text-[15px] whitespace-nowrap text-ink-muted"
        >
          <BrandIcon name={item.icon} className="size-6" />
          {item.name}
        </li>
      ))}
    </ul>
  );
}

/** A slow, endless band of the stacks people bring. Hovering pauses it. */
export function StackMarquee() {
  return (
    <section aria-labelledby="stack-title" className="py-10">
      <h2 id="stack-title" className="text-label text-center text-ink-dim">
        Bring your stack: if it builds into a container, it runs on Brume
      </h2>
      <div className="mt-7 overflow-hidden [mask-image:linear-gradient(to_right,transparent,black_12%,black_88%,transparent)]">
        <div className="flex w-max animate-marquee hover:[animation-play-state:paused] motion-reduce:animate-none">
          <StackList />
          {/* The copy makes the loop seamless: the track moves by half its width. */}
          <StackList hidden />
        </div>
      </div>
    </section>
  );
}
