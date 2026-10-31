import { describe, expect, it } from "vite-plus/test";

import type {
  Environment,
  EnvironmentId,
  ProjectId,
  Service,
  ServiceId,
  ServiceKind,
  Variable,
  VariableId,
  VariableKind,
  VariableLevel,
} from "~/api/types";

import { DecodeError } from "./decode";
import {
  parseVariableKind,
  parseVariableName,
  parseVariables,
  resolveVariables,
  variableCountLabel,
  variableWarnings,
} from "./variables";

const environment: Environment = {
  id: "e-prod" as EnvironmentId,
  projectId: "p-1" as ProjectId,
  name: "production",
};
const service = (kind: ServiceKind = "github"): Service => ({
  id: "s-api" as ServiceId,
  environmentId: environment.id,
  name: "api",
  slug: "api",
  kind,
  status: "online",
  domains: [],
  volume: null,
  dependencies: [],
});
const OWNERS: Record<VariableLevel, string> = {
  project: "p-1",
  environment: "e-prod",
  service: "s-api",
};
const makeVariable = (
  level: VariableLevel,
  name: string,
  kind: VariableKind = "plain",
  value: string | null = `${level}-${name}`,
): Variable => ({
  id: `${level}-${name}` as VariableId,
  level,
  ownerId: OWNERS[level],
  name,
  kind,
  value: kind === "secret" ? null : value,
});

describe("parseVariables", () => {
  const raw = {
    id: "v-1",
    level: "service",
    ownerId: "s-1",
    name: "PORT",
    kind: "plain",
    value: "3000",
  };

  it("decodes the API response", () => {
    expect(parseVariables([raw])).toEqual([raw]);
  });

  it("rejects an unknown level or kind", () => {
    expect(() => parseVariables([{ ...raw, level: "global" }])).toThrow(DecodeError);
    expect(() => parseVariables([{ ...raw, kind: "env" }])).toThrow(DecodeError);
  });
});

describe("resolveVariables", () => {
  it("keeps the most precise level for each name", () => {
    const resolved = resolveVariables(
      [
        makeVariable("project", "LOG_LEVEL"),
        makeVariable("environment", "LOG_LEVEL"),
        makeVariable("project", "APP_URL"),
        makeVariable("service", "APP_URL"),
        makeVariable("project", "REGION"),
      ],
      service(),
      environment,
    );

    expect(resolved.map(({ variable, origin }) => [variable.name, origin])).toEqual([
      ["APP_URL", "service"],
      ["LOG_LEVEL", "environment"],
      ["REGION", "project"],
    ]);
  });

  it("ignores the variables of other owners", () => {
    const other = { ...makeVariable("service", "PORT"), ownerId: "s-other" };

    expect(resolveVariables([other], service(), environment)).toEqual([]);
  });

  it("does not hand shared secrets to every service", () => {
    const resolved = resolveVariables(
      [
        makeVariable("project", "STRIPE_KEY", "secret"),
        makeVariable("service", "API_TOKEN", "secret"),
      ],
      service(),
      environment,
    );

    expect(resolved.map(({ variable }) => variable.name)).toEqual(["API_TOKEN"]);
  });

  it("gives a template database its own settings only", () => {
    const resolved = resolveVariables(
      [makeVariable("project", "LOG_LEVEL"), makeVariable("service", "MAXMEMORY")],
      service("redis"),
      environment,
    );

    expect(resolved.map(({ variable }) => variable.name)).toEqual(["MAXMEMORY"]);
  });

  it("gives a static site the shared build variables only", () => {
    const resolved = resolveVariables(
      [makeVariable("project", "LOG_LEVEL"), makeVariable("project", "PUBLIC_API_URL", "build")],
      service("static"),
      environment,
    );

    expect(resolved.map(({ variable }) => variable.name)).toEqual(["PUBLIC_API_URL"]);
  });
});

describe("parseVariableName", () => {
  it.each(["PORT", "DATABASE_URL", "_PRIVATE", "api_key2"])("accepts %s", (raw) => {
    expect(parseVariableName(raw)).toBe(raw);
  });

  it.each(["", "2FA", "MY-VAR", "A B", "X".repeat(129)])("rejects %j", (raw) => {
    expect(parseVariableName(raw)).toBeUndefined();
  });
});

describe("variableWarnings", () => {
  it("warns when a plain variable looks like a secret", () => {
    for (const name of ["STRIPE_KEY", "GITHUB_TOKEN", "JWT_SECRET", "DB_PASSWORD"]) {
      expect(variableWarnings(name, "x", "plain")).toHaveLength(1);
    }
  });

  it("warns when a value carries credentials", () => {
    expect(
      variableWarnings("MONGO_URL", "mongodb://app:hunter2@mongo:27017", "build"),
    ).toHaveLength(1);
  });

  it("stays quiet for secrets and ordinary values", () => {
    expect(variableWarnings("STRIPE_KEY", "sk_live", "secret")).toEqual([]);
    expect(variableWarnings("API_URL", "https://api.brume.dev", "plain")).toEqual([]);
  });
});

describe("variableCountLabel", () => {
  it.each([
    [0, "Aucune variable"],
    [1, "1 variable"],
    [14, "14 variables"],
  ])("%i → %s", (count, label) => {
    expect(variableCountLabel(count)).toBe(label);
  });
});

describe("parseVariableKind", () => {
  it("keeps a known kind and drops anything else", () => {
    expect(parseVariableKind("secret")).toBe("secret");
    expect(parseVariableKind("env")).toBeUndefined();
    expect(parseVariableKind(null)).toBeUndefined();
  });
});
