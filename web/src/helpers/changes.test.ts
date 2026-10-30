import { describe, expect, it } from "vite-plus/test";

import type {
  ChangeId,
  EnvironmentId,
  ServiceId,
  ServiceSettings,
  StagedChange,
  Variable,
  VariableId,
} from "~/api/types";

import {
  changeCountLabel,
  changesByService,
  describeChange,
  limitStep,
  parseChanges,
  parseDomain,
  settingChange,
  settingValue,
  stagedFields,
  stagedVariableRows,
  validateSetting,
  withStagedSettings,
} from "./changes";
import { DecodeError } from "./decode";

const target = { environmentId: "e-1" as EnvironmentId, serviceId: "s-api" as ServiceId };
const setting = (field: string, value: unknown, id = `c-${field}`): StagedChange =>
  ({ id: id as ChangeId, ...target, type: "setting", field, value }) as StagedChange;

const settings: ServiceSettings = {
  source: { kind: "github", repository: "planchon/brume", branch: "main", autoDeploy: true },
  startCommand: "pnpm start",
  port: 3000,
  healthcheckPath: "/health",
  cpuLimit: 1,
  memoryLimitMb: 512,
  egress: true,
};

describe("parseChanges", () => {
  it("decodes the three kinds of change", () => {
    const raw = [
      { id: "c-1", ...target, type: "setting", field: "port", value: 8080 },
      { id: "c-2", ...target, type: "setting", field: "domains", value: ["a.dev"] },
      { id: "c-3", ...target, type: "variable", name: "API_KEY", kind: "secret", value: null },
      { id: "c-4", ...target, type: "variable-removal", name: "PORT" },
    ];

    expect(parseChanges(raw)).toEqual(raw);
  });

  it("rejects an unknown field or a malformed value", () => {
    expect(() =>
      parseChanges([{ id: "c", ...target, type: "setting", field: "x", value: 1 }]),
    ).toThrow(DecodeError);
    expect(() =>
      parseChanges([{ id: "c", ...target, type: "setting", field: "port", value: {} }]),
    ).toThrow(DecodeError);
  });
});

describe("settingValue and withStagedSettings", () => {
  it("reads the current value of each field", () => {
    expect(settingValue(settings, [], "branch")).toBe("main");
    expect(settingValue(settings, ["api.brume.dev"], "domains")).toEqual(["api.brume.dev"]);
    expect(settingValue(settings, [], "image")).toBeUndefined();
  });

  it("applies the staged values on top of the current settings", () => {
    const staged = withStagedSettings(
      settings,
      ["api.brume.dev"],
      [setting("branch", "develop"), setting("port", 8080), setting("domains", [])],
    );

    expect(staged.settings.source).toEqual({ ...settings.source, branch: "develop" });
    expect(staged.settings.port).toBe(8080);
    expect(staged.domains).toEqual([]);
  });
});

describe("stagedFields", () => {
  it("lists the fields with a staged value", () => {
    const fields = stagedFields([
      setting("port", 1),
      { id: "v" as ChangeId, ...target, type: "variable-removal", name: "X" },
    ]);

    expect([...fields]).toEqual(["port"]);
  });
});

describe("settingChange", () => {
  it("stages a value that differs from the current one", () => {
    expect(settingChange("port", 8080, 3000, [])).toEqual({ action: "stage" });
  });

  it("unstages when the value comes back to the current one", () => {
    expect(settingChange("port", 3000, 3000, [setting("port", 8080)])).toEqual({
      action: "unstage",
      changeId: "c-port",
    });
  });

  it("does nothing when nothing changes", () => {
    expect(settingChange("port", 3000, 3000, [])).toEqual({ action: "none" });
    expect(settingChange("port", 8080, 3000, [setting("port", 8080)])).toEqual({ action: "none" });
  });

  it("compares lists by content", () => {
    expect(settingChange("domains", ["a.dev"], ["a.dev"], [])).toEqual({ action: "none" });
  });
});

describe("validateSetting", () => {
  it("accepts valid values", () => {
    expect(validateSetting("port", " 8080 ")).toEqual({ value: 8080 });
    expect(validateSetting("branch", "feat/logs")).toEqual({ value: "feat/logs" });
    expect(validateSetting("healthcheckPath", "/ready")).toEqual({ value: "/ready" });
    expect(validateSetting("image", "redis:7")).toEqual({ value: "redis:7" });
    expect(validateSetting("startCommand", "node dist/index.js")).toEqual({
      value: "node dist/index.js",
    });
  });

  it.each([
    ["port", "70000"],
    ["port", "80a"],
    ["branch", "bad branch"],
    ["branch", ""],
    ["healthcheckPath", "ready"],
    ["image", "Not An Image"],
    ["startCommand", "x".repeat(501)],
  ] as const)("rejects %s = %j", (field, raw) => {
    expect(validateSetting(field, raw)).toHaveProperty("error");
  });
});

describe("parseDomain", () => {
  it("lowercases a valid domain and rejects anything else", () => {
    expect(parseDomain(" API.Brume.dev ")).toBe("api.brume.dev");
    expect(parseDomain("https://brume.dev")).toBeUndefined();
  });
});

describe("changeCountLabel", () => {
  it.each([
    [1, "Appliquer 1 changement"],
    [3, "Appliquer 3 changements"],
  ])("%i → %s", (count, label) => {
    expect(changeCountLabel(count)).toBe(label);
  });
});

describe("describeChange", () => {
  it.each([
    [setting("branch", "develop"), "Branche → develop"],
    [setting("autoDeploy", false), "Déploiement automatique → désactivé"],
    [setting("memoryLimitMb", 2048), "Limite mémoire → 2 Go"],
    [setting("domains", []), "Domaines publics → aucun"],
    [
      {
        id: "v" as ChangeId,
        ...target,
        type: "variable",
        name: "TOKEN",
        kind: "secret",
        value: null,
      } as const,
      "TOKEN = ••••••••",
    ],
    [
      { id: "v" as ChangeId, ...target, type: "variable-removal", name: "PORT" } as const,
      "PORT supprimée",
    ],
  ] as const)("%j", (change, label) => {
    expect(describeChange(change)).toBe(label);
  });
});

const variable = (name: string, kind: Variable["kind"] = "plain"): Variable => ({
  id: `v-${name}` as VariableId,
  level: "service",
  ownerId: "s-api",
  name,
  kind,
  value: kind === "secret" ? null : `${name}-value`,
});

describe("stagedVariableRows", () => {
  const resolved = [
    { variable: variable("LOG_LEVEL"), origin: "project" as const },
    { variable: variable("PORT"), origin: "service" as const },
    { variable: variable("TOKEN", "secret"), origin: "service" as const },
  ];

  it("marks the variables a change adds, modifies or removes", () => {
    const rows = stagedVariableRows(resolved, [
      {
        id: "c-1" as ChangeId,
        ...target,
        type: "variable",
        name: "PORT",
        kind: "plain",
        value: "8080",
      },
      { id: "c-2" as ChangeId, ...target, type: "variable-removal", name: "TOKEN" },
      {
        id: "c-3" as ChangeId,
        ...target,
        type: "variable",
        name: "API_URL",
        kind: "build",
        value: "https://x",
      },
    ]);

    expect(rows.map(({ name, draft, value, origin }) => [name, draft, value, origin])).toEqual([
      ["API_URL", "added", "https://x", "service"],
      ["LOG_LEVEL", undefined, "LOG_LEVEL-value", "project"],
      ["PORT", "modified", "8080", "service"],
      ["TOKEN", "removed", null, "service"],
    ]);
  });

  it("keeps the change id to undo it", () => {
    const rows = stagedVariableRows(resolved, [
      { id: "c-2" as ChangeId, ...target, type: "variable-removal", name: "TOKEN" },
    ]);

    expect(rows.find(({ name }) => name === "TOKEN")?.changeId).toBe("c-2");
  });
});

describe("changesByService", () => {
  it("groups the changes by service, in the order of the services", () => {
    const api = { id: "s-api", name: "api" } as never;
    const worker = { id: "s-worker", name: "worker" } as never;
    const portChange = setting("port", 1);
    const workerChange = {
      ...setting("egress", false, "c-w"),
      serviceId: "s-worker" as ServiceId,
    } as StagedChange;

    expect(changesByService([portChange, workerChange], [worker, api])).toEqual([
      { service: worker, changes: [workerChange] },
      { service: api, changes: [portChange] },
    ]);
  });
});

describe("limitStep", () => {
  it("finds the step of a limit, or the closest one below", () => {
    expect(limitStep([0.5, 1, 2], 1)).toBe(1);
    expect(limitStep([0.5, 1, 2], 1.5)).toBe(1);
    expect(limitStep([0.5, 1, 2], 0.1)).toBe(0);
  });
});
