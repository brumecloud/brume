import { describe, expect, it } from "vite-plus/test";

import { findChunkCycle } from "./no-chunk-cycles.ts";

describe("findChunkCycle", () => {
  it("returns undefined for an acyclic graph", () => {
    const graph = new Map([
      ["index.js", ["vendor.js", "route.js"]],
      ["route.js", ["vendor.js"]],
      ["vendor.js", []],
    ]);

    expect(findChunkCycle(graph)).toBeUndefined();
  });

  it("returns the chunks of a cycle, closing on the first one", () => {
    const graph = new Map([
      ["index.js", ["a.js"]],
      ["a.js", ["b.js"]],
      ["b.js", ["a.js"]],
    ]);

    expect(findChunkCycle(graph)).toEqual(["a.js", "b.js", "a.js"]);
  });

  it("finds a chunk that imports itself", () => {
    expect(findChunkCycle(new Map([["a.js", ["a.js"]]]))).toEqual(["a.js", "a.js"]);
  });

  it("ignores imports of chunks outside the graph", () => {
    expect(findChunkCycle(new Map([["a.js", ["external.js"]]]))).toBeUndefined();
  });
});
