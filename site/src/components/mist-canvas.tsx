"use client";

import { useEffect, useRef } from "react";

import { type MistShape, startMist } from "~/lib/mist";

interface MistCanvasProps {
  readonly shape: MistShape;
  readonly seed: number;
  readonly stars?: boolean;
}

/** Decorative mist that covers its parent, which must be `relative isolate overflow-hidden`. */
export function MistCanvas({ shape, seed, stars = false }: MistCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Starts the canvas animation once mounted, and stops it on unmount.
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    return startMist(canvas, { shape, seed, stars });
  }, [shape, seed, stars]);

  return (
    <canvas
      ref={canvasRef}
      aria-hidden="true"
      className="pointer-events-none absolute top-0 left-0 -z-10 [image-rendering:pixelated]"
    />
  );
}
