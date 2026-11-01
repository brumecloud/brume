import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import { parseIngress, publicHost } from "./ingress";

describe("parseIngress", () => {
  it("reads the zone, or null before brume install --domain", () => {
    expect(parseIngress({ zone: "example.com" })).toEqual({ zone: "example.com" });
    expect(parseIngress(null)).toBeNull();
  });

  it("rejects a malformed answer", () => {
    expect(() => parseIngress({})).toThrow(DecodeError);
    expect(() => parseIngress(undefined)).toThrow(DecodeError);
  });
});

describe("publicHost", () => {
  it("finds the host that the zone serves", () => {
    expect(publicHost(["demo.example.com"], "example.com")).toBe("demo.example.com");
    expect(publicHost([], "example.com")).toBeUndefined();
  });

  it("ignores a host outside the zone", () => {
    expect(publicHost(["demo.example.org"], "example.com")).toBeUndefined();
    expect(publicHost(["example.com"], "example.com")).toBeUndefined();
  });
});
