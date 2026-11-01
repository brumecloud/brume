import { MistCanvas } from "./mist-canvas";

interface PageHeroProps {
  readonly title: string;
  readonly lede: string;
  // A short status line above the title, such as the alpha this page describes.
  readonly badge?: string;
}

/** The top of a sub-page: the night sky of the landing, shorter, under the header. */
export function PageHero({ title, lede, badge }: PageHeroProps) {
  return (
    <section
      id="top"
      aria-labelledby="page-title"
      className="relative isolate overflow-hidden pt-[clamp(8rem,20vh,11rem)] pb-28"
    >
      <MistCanvas shape="hero" seed={5} stars />
      <div className="mx-auto w-full max-w-page px-6">
        {badge ? <p className="text-label text-ink-dim">{badge}</p> : null}
        <h1
          id="page-title"
          className="mt-4 max-w-[18ch] text-[clamp(2.5rem,7vw,5rem)] leading-[1] font-semibold tracking-[-0.045em] text-balance"
        >
          {title}
        </h1>
        <p className="mt-6 max-w-[40rem] text-lg leading-relaxed text-pretty text-ink-muted">
          {lede}
        </p>
      </div>
    </section>
  );
}
