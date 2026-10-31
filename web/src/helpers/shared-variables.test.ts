import { describe, expect, it } from "vite-plus/test";

import type {
  Environment,
  EnvironmentId,
  ProjectId,
  Service,
  ServiceId,
  Variable,
  VariableId,
  VariableKind,
  VariableLevel,
} from "~/api/types";

import { DecodeError } from "./decode";
import {
  groupImpact,
  impactConfirmLabel,
  impactSummary,
  parseImpact,
  parseRemoval,
  parseSharedVariableResult,
  referenceableSecrets,
  referencesImpact,
  sharedVariableWarnings,
  sharedVariables,
} from "./shared-variables";

const projectId = "p-1" as ProjectId;
const environment = (name: string): Environment => ({
  id: `e-${name}` as EnvironmentId,
  projectId,
  name,
});
const production = environment("production");
const staging = environment("staging");
const preview = environment("preview");

const service = (name: string, env: Environment): Service => ({
  id: `${env.name}-${name}` as ServiceId,
  environmentId: env.id,
  name,
  slug: name,
  kind: "image",
  status: "online",
  domains: [],
  volume: null,
  dependencies: [],
});

const variable = (
  level: VariableLevel,
  ownerId: string,
  name: string,
  kind: VariableKind = "plain",
): Variable => ({
  id: `${ownerId}-${name}` as VariableId,
  level,
  ownerId,
  name,
  kind,
  value: kind === "secret" ? null : "value",
  reference: false,
});

const touched = (env: Environment, name: string) => ({
  serviceId: `${env.name}-${name}` as ServiceId,
  environmentId: env.id,
});

describe("groupImpact", () => {
  const services = [
    service("worker", production),
    service("api", production),
    service("api", staging),
    service("web", preview),
  ];

  it("groups the services by environment, production first", () => {
    expect(
      groupImpact(
        [
          touched(staging, "api"),
          touched(production, "worker"),
          touched(preview, "web"),
          touched(production, "api"),
        ],
        [staging, preview, production],
        services,
      ),
    ).toEqual([
      { environmentName: "production", services: ["api", "worker"] },
      { environmentName: "preview", services: ["web"] },
      { environmentName: "staging", services: ["api"] },
    ]);
  });

  it("skips what it cannot name", () => {
    expect(
      groupImpact(
        [{ serviceId: "gone" as ServiceId, environmentId: production.id }],
        [production],
        services,
      ),
    ).toEqual([]);
  });
});

describe("impactSummary", () => {
  it.each([
    [0, "This change redeploys no services."],
    [1, "This change redeploys 1 service."],
    [3, "This change redeploys 3 services."],
  ])("%i", (count, summary) => {
    expect(impactSummary(count)).toBe(summary);
  });
});

describe("impactConfirmLabel", () => {
  it.each([
    [0, "Save"],
    [1, "Redeploy 1 service"],
    [3, "Redeploy 3 services"],
  ])("%i", (count, label) => {
    expect(impactConfirmLabel(count)).toBe(label);
  });
});

describe("referencesImpact", () => {
  it("locates the services that reference a secret", () => {
    const api = service("api", staging);

    expect(
      referencesImpact([api.id, "gone" as ServiceId], [api, service("web", production)]),
    ).toEqual([{ serviceId: api.id, environmentId: staging.id }]);
  });
});

describe("sharedVariableWarnings", () => {
  it("reminds that a project secret also reaches staging", () => {
    expect(sharedVariableWarnings("project", "STRIPE_KEY", "sk", "secret")).toEqual([
      "Project secrets also reach staging. Consider defining it per environment.",
    ]);
  });

  it("does not for an environment secret", () => {
    expect(sharedVariableWarnings("environment", "STRIPE_KEY", "sk", "secret")).toEqual([]);
  });

  it("keeps the warnings of a plain variable", () => {
    expect(sharedVariableWarnings("project", "STRIPE_KEY", "sk", "plain")).toHaveLength(1);
  });
});

describe("sharedVariables", () => {
  it("keeps one level of one owner, sorted by name", () => {
    const variables = [
      variable("project", projectId, "TZ"),
      variable("environment", production.id, "APP_URL"),
      variable("project", projectId, "LOG_LEVEL"),
      variable("project", "p-other", "REGION"),
    ];

    expect(sharedVariables(variables, "project", projectId).map((shared) => shared.name)).toEqual([
      "LOG_LEVEL",
      "TZ",
    ]);
  });
});

describe("referenceableSecrets", () => {
  const variables = [
    variable("project", projectId, "SENTRY_DSN", "secret"),
    variable("project", projectId, "STRIPE_KEY", "secret"),
    variable("environment", production.id, "STRIPE_KEY", "secret"),
    variable("environment", production.id, "MAILER_TOKEN", "secret"),
    variable("environment", staging.id, "STAGING_TOKEN", "secret"),
    variable("project", projectId, "LOG_LEVEL"),
  ];

  it("lists the environment's secrets, then the project's, once each", () => {
    expect(referenceableSecrets(variables, production, [])).toEqual([
      "MAILER_TOKEN",
      "STRIPE_KEY",
      "SENTRY_DSN",
    ]);
  });

  it("leaves out the names the service already has", () => {
    expect(referenceableSecrets(variables, production, ["STRIPE_KEY"])).toEqual([
      "MAILER_TOKEN",
      "SENTRY_DSN",
    ]);
  });
});

describe("decoding", () => {
  const raw = {
    id: "v-1",
    level: "project",
    ownerId: "p-1",
    name: "TZ",
    kind: "plain",
    value: "UTC",
    reference: false,
  };
  const deployment = {
    id: "d-1",
    serviceId: "s-1",
    status: "deploying",
    createdAt: "2026-10-31T10:00:00.000Z",
    author: "admin",
    commit: null,
    image: "redis:8",
    rollbackable: true,
  };

  it("decodes a saved shared variable", () => {
    expect(
      parseSharedVariableResult({
        variable: raw,
        deployments: [deployment],
        referencedBy: ["s-2"],
      }),
    ).toEqual({ variable: raw, deployments: [deployment], referencedBy: ["s-2"] });
  });

  it("decodes a removal and an impact", () => {
    expect(parseRemoval({ deployments: [] })).toEqual({ deployments: [] });
    expect(parseImpact({ services: [{ serviceId: "s-1", environmentId: "e-1" }] })).toEqual({
      services: [{ serviceId: "s-1", environmentId: "e-1" }],
    });
  });

  it("rejects a malformed impact", () => {
    expect(() => parseImpact({ services: [{ serviceId: 1 }] })).toThrow(DecodeError);
  });
});
