const MIN_DELAY_BETWEEN_RELOADS_MS = 10_000;

// After a Brume update the old chunks are gone: reload the page once, without
// looping if the reload does not fix it.
export function shouldReloadAfterPreloadError(lastReloadAt: string | null, now: number): boolean {
  const last = Number(lastReloadAt);
  return !Number.isFinite(last) || now - last > MIN_DELAY_BETWEEN_RELOADS_MS;
}
