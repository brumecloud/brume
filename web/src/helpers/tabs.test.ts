import { describe, expect, it } from "vite-plus/test";

import { HIDDEN_PILL, pillHidden, pillOnTab } from "./tabs";

describe("pillOnTab", () => {
  it("appears on the first hovered tab without sliding", () => {
    expect(pillOnTab(HIDDEN_PILL, 12, 80)).toEqual({
      left: 12,
      width: 80,
      visible: true,
      slide: false,
    });
  });

  it("slides from a hovered tab to the next", () => {
    expect(pillOnTab(pillOnTab(HIDDEN_PILL, 12, 80), 92, 60)).toEqual({
      left: 92,
      width: 60,
      visible: true,
      slide: true,
    });
  });
});

describe("pillHidden", () => {
  it("fades where it is, and appears again without sliding", () => {
    const hidden = pillHidden(pillOnTab(HIDDEN_PILL, 12, 80));

    expect(hidden).toEqual({ left: 12, width: 80, visible: false, slide: false });
    expect(pillOnTab(hidden, 92, 60).slide).toBe(false);
  });
});
