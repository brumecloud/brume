// Which block the reading line, the middle of the screen, lights up.

interface Box {
  readonly top: number;
  readonly bottom: number;
  readonly left: number;
}

/**
 * The index of the block to light, or -1. A block the line crosses wins. Blocks side by side in a
 * row light one after the other, left to right, as the line goes down the row. When the line falls
 * between blocks, the nearest one stays lit, up to `reach` away: the last block of a row the line
 * has left, the first of a row it nears.
 */
export function blockToLight(boxes: readonly Box[], line: number, reach: number): number {
  const crossed = boxes
    .map((box, index) => ({ box, index }))
    .filter(({ box }) => box.top <= line && line <= box.bottom);
  if (crossed.length > 0) {
    const row = crossed.toSorted((a, b) => a.box.left - b.box.left);
    const top = Math.min(...row.map(({ box }) => box.top));
    const bottom = Math.max(...row.map(({ box }) => box.bottom));
    const progress = bottom > top ? (line - top) / (bottom - top) : 0;
    return row[Math.min(row.length - 1, Math.floor(progress * row.length))]?.index ?? -1;
  }
  const away = (box: Box) => Math.min(Math.abs(box.top - line), Math.abs(box.bottom - line));
  const distance = Math.min(...boxes.map(away));
  if (!(distance < reach)) {
    return -1;
  }
  // A row the line has just left keeps its last block lit; a row it nears lights its first.
  const nearest = boxes
    .map((box, index) => ({ box, index }))
    .filter(({ box }) => away(box) - distance < 1)
    .toSorted((a, b) => a.box.left - b.box.left);
  const above = (nearest[0]?.box.bottom ?? 0) < line;
  return (above ? nearest.at(-1) : nearest[0])?.index ?? -1;
}

/**
 * The block a pinned grid lights, from how far the grid has travelled through the scroll it holds
 * for, or -1 when the line is outside it or the grid does not pin (no scroll to hold for).
 */
export function pinnedBlockToLight(
  track: Pick<Box, "top" | "bottom">,
  grid: { readonly top: number; readonly height: number },
  count: number,
  line: number,
): number {
  const travel = track.bottom - track.top - grid.height;
  if (count === 0 || travel <= 1 || line < track.top || line > track.bottom) {
    return -1;
  }
  const progress = Math.min(1, Math.max(0, (grid.top - track.top) / travel));
  return Math.min(count - 1, Math.floor(progress * count));
}
