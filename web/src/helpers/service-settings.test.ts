import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import { parseServiceSettings } from "./service-settings";

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
