"use client";

import { type ReactNode, useEffect, useRef, useState } from "react";

type PlayState = "idle" | "waiting" | "playing";

interface PlayOnViewProps {
  readonly className?: string;
  readonly children: ReactNode;
}

/**
 * Sets `data-play` to "playing" the first time the block is on screen, so CSS
 * can start its animations then. Without JavaScript, nothing is hidden.
 */
export function PlayOnView({ className = "", children }: PlayOnViewProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [state, setState] = useState<PlayState>("idle");

  // Waits for the block to scroll into view, then plays its animations once.
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    setState("waiting");
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry?.isIntersecting) return;
        setState("playing");
        observer.disconnect();
      },
      { threshold: 0.4 },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={ref} data-play={state} className={className}>
      {children}
    </div>
  );
}
