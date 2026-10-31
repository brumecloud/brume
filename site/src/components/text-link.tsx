import type { ReactNode } from "react";

interface TextLinkProps {
  readonly href: string;
  readonly children: ReactNode;
}

/** A secondary link, underlined with dashes. */
export function TextLink({ href, children }: TextLinkProps) {
  return (
    <a
      href={href}
      className="border-b border-dashed border-edge-strong pb-0.5 transition-colors hover:text-brand"
    >
      {children}
    </a>
  );
}
