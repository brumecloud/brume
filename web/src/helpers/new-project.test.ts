import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import {
  parseCreatedProject,
  parseImageReference,
  parseNewProjectSearch,
  parseRepositories,
} from "./new-project";

describe("parseNewProjectSearch", () => {
  it.each(["github", "static", "database", "image"])("keeps the %s step", (step) => {
    expect(parseNewProjectSearch({ step })).toEqual({ step });
  });

  it("drops an unknown step", () => {
    expect(parseNewProjectSearch({ step: "../admin" })).toEqual({});
  });
});

describe("parseImageReference", () => {
  it.each([
    "redis",
    "redis:7.4",
    "ghcr.io/planchon/brume:latest",
    "localhost:5000/team/app",
    `nginx@sha256:${"a".repeat(64)}`,
  ])("accepts %s", (raw) => {
    expect(parseImageReference(raw)).toBe(raw);
  });

  it("trims spaces around the reference", () => {
    expect(parseImageReference("  redis:7 ")).toBe("redis:7");
  });

  it.each(["", "Redis", "redis:", "-redis", "a b", "redis;rm -rf", "x".repeat(256)])(
    "rejects %j",
    (raw) => {
      expect(parseImageReference(raw)).toBeUndefined();
    },
  );
});

describe("parseRepositories", () => {
  it("decodes the repository list", () => {
    expect(parseRepositories([{ fullName: "planchon/brume" }])).toEqual([
      { fullName: "planchon/brume" },
    ]);
  });

  it("rejects a malformed repository", () => {
    expect(() => parseRepositories([{ name: "brume" }])).toThrow(DecodeError);
  });
});

describe("parseCreatedProject", () => {
  it("decodes the created project", () => {
    expect(parseCreatedProject({ projectId: "p-1", environmentName: "production" })).toEqual({
      projectId: "p-1",
      environmentName: "production",
    });
  });
});
