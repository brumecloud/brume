import type { ReactNode } from "react";

interface SectionProps {
  readonly id: string;
  readonly title: string;
  readonly lede?: ReactNode;
  readonly className?: string;
  readonly children?: ReactNode;
}

/** A page section with its title and lede. */
export function Section({ id, title, lede, className = "", children }: SectionProps) {
  return (
    <section id={id} aria-labelledby={`${id}-title`} className={`scroll-mt-8 py-24 ${className}`}>
      <div className="mx-auto w-full max-w-page px-6">
        <SectionHeading id={`${id}-title`} title={title} lede={lede} />
        {children}
      </div>
    </section>
  );
}

interface SectionHeadingProps {
  readonly id: string;
  readonly title: string;
  readonly lede?: ReactNode;
  readonly centered?: boolean;
}

export function SectionHeading({ id, title, lede, centered = false }: SectionHeadingProps) {
  return (
    <div className={`reveal ${centered ? "flex flex-col items-center text-center" : ""}`}>
      <h2
        id={id}
        className="max-w-[22ch] text-[clamp(1.875rem,3.8vw,2.75rem)] leading-[1.1] font-semibold tracking-[-0.035em] text-balance"
      >
        {title}
      </h2>
      {lede ? (
        <p className="mt-5 max-w-[60ch] text-[17px] leading-relaxed text-pretty text-ink-muted">
          {lede}
        </p>
      ) : null}
    </div>
  );
}
