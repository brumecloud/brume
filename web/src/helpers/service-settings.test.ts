import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import type { Service, ServiceKind } from "~/api/types";

import { currentSection, parseServiceSettings, settingsSections } from "./service-settings";

const raw = {
  source: { kind: "github", repository: "planchon/brume", branch: "main", autoDeploy: true },
  startCommand: "pnpm start",
  port: 3000,
  healthcheckPath: "/health",
  cpuLimit: 1,
  memoryLimitMb: 512,
  egress: true,
};

describe("parseServiceSettings", () => {
  it("decodes the settings of a GitHub service", () => {
    expect(parseServiceSettings(raw)).toEqual(raw);
  });

  it("decodes the settings of a template database", () => {
    const template = {
      ...raw,
      source: { kind: "template", template: "postgres", version: "17.6" },
      startCommand: null,
      port: null,
      healthcheckPath: null,
      egress: false,
    };

    expect(parseServiceSettings(template)).toEqual(template);
  });

  it("rejects an unknown source or a malformed limit", () => {
    expect(() => parseServiceSettings({ ...raw, source: { kind: "ftp" } })).toThrow(DecodeError);
    expect(() => parseServiceSettings({ ...raw, cpuLimit: "1" })).toThrow(DecodeError);
  });
});

const service = (kind: ServiceKind, volume: Service["volume"] = null) =>
  ({ kind, volume }) as Service;

describe("settingsSections", () => {
  it("shows every section of an app with a volume", () => {
    expect(settingsSections(service("github", { name: "data", mountPath: "/data" }))).toEqual([
      "source",
      "network",
      "deploy",
      "resources",
      "volume",
      "danger",
    ]);
  });

  it("drops the command and the limits of a static site, served by the shared nginx", () => {
    expect(settingsSections(service("static"))).toEqual(["source", "network", "danger"]);
  });

  it("drops the command of a template database", () => {
    expect(settingsSections(service("postgres"))).toEqual([
      "source",
      "network",
      "resources",
      "danger",
    ]);
  });
});

const positions = (tops: readonly number[]) =>
  (["source", "network", "deploy"] as const).map((id, index) => ({ id, top: tops[index] ?? 0 }));

describe("currentSection", () => {
  it("is the first section before any scroll", () => {
    expect(currentSection(positions([24, 600, 1200]), false)).toBe("source");
  });

  it("is the last section whose title reached the top", () => {
    expect(currentSection(positions([-700, 100, 600]), false)).toBe("network");
  });

  it("is the last section at the bottom of the page", () => {
    expect(currentSection(positions([-1200, -600, 300]), true)).toBe("deploy");
  });

  it("is undefined without sections", () => {
    expect(currentSection([], false)).toBeUndefined();
  });
});
