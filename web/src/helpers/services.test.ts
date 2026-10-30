import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import { parseServices } from "./services";

describe("parseServices", () => {
  const raw = {
    id: "s-1",
    environmentId: "e-1",
    name: "brume-api",
    slug: "brume-api",
    kind: "github",
    status: "online",
    domains: ["api.brume.dev"],
    volume: null,
    dependencies: ["s-2"],
  };

  it("decodes the API response", () => {
    expect(parseServices([raw])).toEqual([raw]);
  });

  it("decodes an attached volume", () => {
    const volume = { name: "mongodb-data", mountPath: "/data/db" };

    expect(parseServices([{ ...raw, volume }])[0]?.volume).toEqual(volume);
  });

  it("rejects malformed domains, volume or dependencies", () => {
    expect(() => parseServices([{ ...raw, domains: "api.brume.dev" }])).toThrow(DecodeError);
    expect(() => parseServices([{ ...raw, volume: { name: "data" } }])).toThrow(DecodeError);
    expect(() => parseServices([{ ...raw, dependencies: [3] }])).toThrow(DecodeError);
  });

  it("rejects an unknown kind or status", () => {
    expect(() => parseServices([{ ...raw, kind: "lambda" }])).toThrow(DecodeError);
    expect(() => parseServices([{ ...raw, status: "sleeping" }])).toThrow(DecodeError);
  });
});
