import type { PropsWithChildren, ReactNode } from "react";

type AccordionsProps = PropsWithChildren<{
  type?: "single" | "multiple";
}>;

export function Accordions({ type = "single", children }: AccordionsProps) {
  return (
    <section className="brume-accordions" data-brume-accordions data-type={type}>
      {children}
    </section>
  );
}

export function Accordion({
  title,
  id,
  children,
}: PropsWithChildren<{
  title: ReactNode;
  id?: string;
}>) {
  return (
    <details className="brume-accordion" id={id}>
      <summary>
        <span>{title}</span>
        <span aria-hidden="true" className="brume-accordion-chevron" />
      </summary>
      <div className="brume-accordion-content">
        <div>{children}</div>
      </div>
    </details>
  );
}
