"use client";

import { useEffect } from "react";

import { blockToLight, pinnedBlockToLight } from "~/lib/reading-light";

/**
 * Lights the block in the middle of the screen, so its colors follow the reading as the page
 * scrolls. A block opts in with `data-light`; the lit one gets `data-lit`, which the `lit` variant
 * in globals.css colors. Inside a pinned grid (PinnedGrid), the scroll lights the blocks in turn.
 */
export function ReadingLight() {
  // Follows the scroll and the size of the window, one measure per frame at most.
  useEffect(() => {
    let frame = 0;
    let lit: Element | undefined;
    const update = () => {
      frame = 0;
      const line = window.innerHeight / 2;
      const next = litInPinnedGrid(line) ?? litOnTheLine(line);
      if (next === lit) return;
      lit?.removeAttribute("data-lit");
      next?.setAttribute("data-lit", "");
      lit = next;
    };
    const schedule = () => {
      frame ||= requestAnimationFrame(update);
    };
    // A pinned grid centers itself from its own height.
    const grids = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const height = entry.borderBoxSize[0]?.blockSize ?? entry.contentRect.height;
        (entry.target as HTMLElement).style.setProperty("--pin-height", `${height}px`);
      }
      schedule();
    });
    for (const grid of document.querySelectorAll("[data-pin] > .pin-grid")) grids.observe(grid);
    update();
    window.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    return () => {
      cancelAnimationFrame(frame);
      grids.disconnect();
      window.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      lit?.removeAttribute("data-lit");
    };
  }, []);

  return null;
}

function litInPinnedGrid(line: number): Element | undefined {
  for (const track of document.querySelectorAll("[data-pin]")) {
    const grid = track.querySelector(".pin-grid");
    if (!grid) continue;
    const blocks = grid.querySelectorAll("[data-light]");
    const box = grid.getBoundingClientRect();
    const index = pinnedBlockToLight(
      track.getBoundingClientRect(),
      { top: box.top, height: box.height },
      blocks.length,
      line,
    );
    if (index >= 0) return blocks[index];
  }
  return undefined;
}

function litOnTheLine(line: number): Element | undefined {
  const blocks = [...document.querySelectorAll("[data-light]")];
  const boxes = blocks.map((block) => block.getBoundingClientRect());
  return blocks[blockToLight(boxes, line, window.innerHeight / 4)];
}
