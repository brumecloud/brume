import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import { domainUrl, formatLimits, parseServices, serviceNames } from "./services";

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

describe("domainUrl", () => {
  it("builds an https link to a public domain", () => {
    expect(domainUrl("api.brume.dev")).toBe("https://api.brume.dev/");
  });

  it.each(["data:text/html,x", "evil.com/@x", "a b", "", "x".repeat(254)])("refuses %j", (raw) => {
    expect(domainUrl(raw)).toBeUndefined();
  });
});

describe("formatLimits", () => {
  it("formats the CPU and memory limits", () => {
    expect(formatLimits(1, 512)).toBe("1 vCPU · 512 Mo");
    expect(formatLimits(0.5, 2048)).toBe("0,5 vCPU · 2 Go");
  });
});

describe("serviceNames", () => {
  it("maps each service id to its name", () => {
    const [api] = parseServices([
      {
        id: "s-1",
        environmentId: "e-1",
        name: "api",
        slug: "api",
        kind: "github",
        status: "online",
        domains: [],
        volume: null,
        dependencies: [],
      },
    ]);

    expect(serviceNames(api ? [api] : [])).toEqual(new Map([["s-1", "api"]]));
  });
});
