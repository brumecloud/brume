import { describe, expect, it } from "vite-plus/test";

import type { Environment, EnvironmentId, ProjectId } from "~/api/types";

import { DecodeError } from "./decode";
import {
  defaultEnvironment,
  findEnvironment,
  parseEnvironmentName,
  parseEnvironments,
  parseProjectId,
  parseProjects,
} from "./projects";

const brume = "p-brume" as ProjectId;
const env = (name: string, projectId = brume): Environment => ({
  id: `${projectId}-${name}` as EnvironmentId,
  projectId,
  name,
});

describe("parseProjects", () => {
  it("decodes the API response", () => {
    const raw = [
      { id: "p-brume", name: "brume", description: "", updatedAt: "2026-10-28T10:00:00Z" },
    ];

    expect(parseProjects(raw)).toEqual(raw);
  });

  it("rejects a project without a name", () => {
    expect(() => parseProjects([{ id: "p-brume", description: "", updatedAt: "" }])).toThrow(
      DecodeError,
    );
  });
});

describe("parseEnvironments", () => {
  it("decodes the API response", () => {
    const raw = [{ id: "e-1", projectId: "p-brume", name: "production" }];

    expect(parseEnvironments(raw)).toEqual(raw);
  });
});

describe("parseProjectId", () => {
  it("accepts an id made of letters, digits, dashes and underscores", () => {
    expect(parseProjectId("c0a1d9f0-fd9b-4683-b608-2bf60613b499")).toBe(
      "c0a1d9f0-fd9b-4683-b608-2bf60613b499",
    );
  });

  it.each(["", "../admin", "a b", "x".repeat(65), "<script>"])("rejects %j", (raw) => {
    expect(parseProjectId(raw)).toBeUndefined();
  });
});

describe("parseEnvironmentName", () => {
  it.each(["production", "staging", "pr-12", "a"])("accepts %j", (raw) => {
    expect(parseEnvironmentName(raw)).toBe(raw);
  });

  it.each(["", "Prod", "-prod", "prod-", "pro_d", "x".repeat(64)])("rejects %j", (raw) => {
    expect(parseEnvironmentName(raw)).toBeUndefined();
  });
});

describe("defaultEnvironment", () => {
  it("prefers production", () => {
    expect(defaultEnvironment([env("dev"), env("production")], brume)?.name).toBe("production");
  });

  it("falls back to the first environment by name", () => {
    expect(defaultEnvironment([env("staging"), env("dev")], brume)?.name).toBe("dev");
  });

  it("ignores the environments of other projects", () => {
    const other = "p-other" as ProjectId;

    expect(defaultEnvironment([env("production", other), env("dev")], brume)?.name).toBe("dev");
    expect(defaultEnvironment([env("production", other)], brume)).toBeUndefined();
  });
});

describe("findEnvironment", () => {
  it("finds an environment of the project by name", () => {
    const other = "p-other" as ProjectId;
    const environments = [env("production", other), env("production"), env("dev")];

    expect(findEnvironment(environments, brume, "production")).toBe(environments[1]);
    expect(findEnvironment(environments, brume, "staging")).toBeUndefined();
  });
});
