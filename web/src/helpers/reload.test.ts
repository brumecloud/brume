import { describe, expect, it } from "vite-plus/test";

import { shouldReloadAfterPreloadError } from "./reload";

describe("shouldReloadAfterPreloadError", () => {
  const now = 1_000_000;

  it("reloads when the tab never reloaded for this reason", () => {
    expect(shouldReloadAfterPreloadError(null, now)).toBe(true);
  });

  it("does not reload again right after a reload", () => {
    expect(shouldReloadAfterPreloadError(String(now - 5_000), now)).toBe(false);
  });

  it("reloads again once the previous reload is old", () => {
    expect(shouldReloadAfterPreloadError(String(now - 60_000), now)).toBe(true);
  });

  it("treats a corrupted flag as no previous reload", () => {
    expect(shouldReloadAfterPreloadError("abc", now)).toBe(true);
  });
});
