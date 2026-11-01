import { describe, expect, it } from "vite-plus/test";

import type {
  ActivityEvent,
  Environment,
  EnvironmentId,
  ProjectId,
  Service,
  ServiceId,
} from "~/api/types";

import { DecodeError } from "./decode";
import { activityEntity, activityRows, activityTone, parseActivityPage } from "./activity";

const event = (seq: number, overrides: Partial<ActivityEvent> = {}): ActivityEvent => ({
  seq,
  at: "2026-10-31T10:00:00.000Z",
  type: "ProjectCreated",
  actor: "admin",
  environmentId: null,
  serviceId: null,
  deploymentId: null,
  summary: "Created project northbean",
  ...overrides,
});

describe("activityTone", () => {
  it.each([
    ["DeploymentSucceeded", "success"],
    ["DeploymentFailed", "danger"],
    ["LoginFailed", "danger"],
    ["DeploymentStarted", "info"],
    ["DeploymentRequested", "info"],
    ["ProjectCreated", "neutral"],
    ["SecretRevealed", "neutral"],
  ] as const)("%s is %s", (type, tone) => {
    expect(activityTone(type)).toBe(tone);
  });
});

describe("parseActivityPage", () => {
  it("decodes a page of events", () => {
    const raw = { events: [event(3, { serviceId: "s-1" as ServiceId })], nextBefore: 3 };

    expect(parseActivityPage(raw)).toEqual(raw);
  });

  it("decodes the last page", () => {
    expect(parseActivityPage({ events: [], nextBefore: null })).toEqual({
      events: [],
      nextBefore: null,
    });
  });

  it("rejects an event without a summary", () => {
    expect(() =>
      parseActivityPage({ events: [{ ...event(1), summary: undefined }], nextBefore: null }),
    ).toThrow(DecodeError);
  });
});

describe("activityRows", () => {
  it("puts the pages back in log order, once each", () => {
    const pages = [
      { events: [event(9), event(8)], nextBefore: 8 },
      { events: [event(8), event(2)], nextBefore: null },
    ];

    expect(activityRows(pages).map(({ seq }) => seq)).toEqual([2, 8, 9]);
  });
});

describe("activityEntity", () => {
  const production: Environment = {
    id: "e-prod" as EnvironmentId,
    projectId: "p-1" as ProjectId,
    name: "production",
  };
  const api: Service = {
    id: "s-api" as ServiceId,
    environmentId: production.id,
    name: "api",
    slug: "api",
    kind: "image",
    status: "online",
    domains: [],
    volume: null,
    dependencies: [],
  };

  it("names the service and its environment", () => {
    expect(
      activityEntity(
        event(1, { environmentId: production.id, serviceId: api.id }),
        [production],
        [api],
      ),
    ).toBe("api · production");
  });

  it("names the environment alone", () => {
    expect(activityEntity(event(1, { environmentId: production.id }), [production], [api])).toBe(
      "production",
    );
  });

  it("names the environment of a deleted service", () => {
    expect(
      activityEntity(
        event(1, { environmentId: production.id, serviceId: "gone" as ServiceId }),
        [production],
        [],
      ),
    ).toBe("production");
  });

  it("has nothing to name for a project event or a service it cannot place", () => {
    expect(activityEntity(event(1), [production], [api])).toBeNull();
    expect(
      activityEntity(event(1, { serviceId: "gone" as ServiceId }), [production], []),
    ).toBeNull();
  });
});
