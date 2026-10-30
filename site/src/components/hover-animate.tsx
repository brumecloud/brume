"use client";

import { AnimateIcon } from "@animate-ui/react/icons";
import type { ReactNode } from "react";

interface HoverAnimateProps {
  readonly as?: "article" | "div";
  readonly className?: string;
  readonly children: ReactNode;
}

/**
 * Plays the animated icons inside the block in a loop while it is hovered.
 *
 * The wrapping element is created here, on the client. Animate UI's `asChild`
 * needs a real element as its child: an element streamed from a server
 * component reaches the client as a lazy reference, which it drops, and the
 * page no longer matches the server HTML.
 */
export function HoverAnimate({ as: Element = "div", className = "", children }: HoverAnimateProps) {
  return (
    <AnimateIcon animateOnHover loop loopDelay={250} asChild>
      <Element className={className}>{children}</Element>
    </AnimateIcon>
  );
}
