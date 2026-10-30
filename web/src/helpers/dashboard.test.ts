import { describe, expect, it } from "vite-plus/test";

import type {
  Environment,
  EnvironmentId,
  Project,
  ProjectId,
  Service,
  ServiceId,
  ServiceStatus,
} from "~/api/types";

import {
  formatDay,
  normalizeQuery,
  parseProjectsSearch,
  projectCountLabel,
  projectHealth,
  projectSummaries,
  servicesOnlineLabel,
} from "./dashboard";

const project = (name: string, updatedAt: string, description = ""): Project => ({
  id: `p-${name}` as ProjectId,
  name,
  description,
  updatedAt,
});
const environment = (projectName: string, name: string): Environment => ({
  id: `e-${projectName}-${name}` as EnvironmentId,
  projectId: `p-${projectName}` as ProjectId,
  name,
});
const service = (env: Environment, name: string, status: ServiceStatus = "online"): Service => ({
  id: `s-${env.id}-${name}` as ServiceId,
  environmentId: env.id,
  name,
  slug: name,
  kind: "github",
  status,
});

const brume = project("brume", "2026-10-29T10:00:00Z", "API et worker");
const eve = project("eve", "2026-10-12T10:00:00Z");
const melled = project("melled", "2026-10-21T10:00:00Z");
const brumeProd = environment("brume", "production");
const brumeStaging = environment("brume", "staging");
const eveDev = environment("eve", "dev");

describe("parseProjectsSearch", () => {
  it("keeps known values", () => {
    expect(parseProjectsSearch({ view: "list", sort: "name", q: "api" })).toEqual({
      view: "list",
      sort: "name",
      q: "api",
    });
  });

  it("drops unknown values and empty searches", () => {
    expect(parseProjectsSearch({ view: "table", sort: 3, q: "" })).toEqual({});
  });

  it("bounds the search text", () => {
    expect(parseProjectsSearch({ q: "x".repeat(500) }).q).toHaveLength(100);
  });
});

describe("projectSummaries", () => {
  const environments = [brumeProd, brumeStaging, eveDev];
  const services = [
    service(brumeProd, "worker"),
    service(brumeProd, "api", "crashed"),
    service(brumeStaging, "api"),
    service(eveDev, "shim"),
  ];

  it("summarizes the default environment of each project", () => {
    const [summary] = projectSummaries([brume], environments, services, {});

    expect(summary?.environment).toBe(brumeProd);
    expect(summary?.services.map((s) => s.name)).toEqual(["api", "worker"]);
    expect(summary?.onlineCount).toBe(1);
  });

  it("handles a project without environments", () => {
    const [summary] = projectSummaries([melled], environments, services, {});

    expect(summary).toEqual({
      project: melled,
      environment: undefined,
      services: [],
      onlineCount: 0,
    });
  });

  it("sorts by most recent activity by default", () => {
    const names = projectSummaries([eve, brume, melled], environments, services, {}).map(
      (s) => s.project.name,
    );

    expect(names).toEqual(["brume", "melled", "eve"]);
  });

  it("sorts by name on request", () => {
    const names = projectSummaries([melled, eve, brume], environments, services, {
      sort: "name",
    }).map((s) => s.project.name);

    expect(names).toEqual(["brume", "eve", "melled"]);
  });

  const search = (q: string) =>
    projectSummaries([brume, eve, melled], environments, services, { q }).map(
      (s) => s.project.name,
    );

  it("filters on the name and the description, ignoring case", () => {
    expect(search("EV")).toEqual(["eve"]);
    expect(search("worker")).toEqual(["brume"]);
    expect(search("  ")).toEqual(["brume", "melled", "eve"]);
  });
});

describe("servicesOnlineLabel", () => {
  it.each([
    [0, 0, "Aucun service"],
    [0, 1, "0/1 service en ligne"],
    [2, 2, "2/2 services en ligne"],
    [4, 5, "4/5 services en ligne"],
  ])("%i of %i → %s", (online, total, label) => {
    expect(servicesOnlineLabel(online, total)).toBe(label);
  });
});

describe("formatDay", () => {
  it("formats a date in French", () => {
    expect(formatDay("2026-10-28T12:00:00Z")).toBe("28 oct. 2026");
  });
});

describe("normalizeQuery", () => {
  it("drops an empty or blank search", () => {
    expect(normalizeQuery("")).toBeUndefined();
    expect(normalizeQuery("   ")).toBeUndefined();
  });

  it("keeps the text as typed, bounded", () => {
    expect(normalizeQuery("brume ")).toBe("brume ");
    expect(normalizeQuery("x".repeat(500))).toHaveLength(100);
  });
});

describe("projectCountLabel", () => {
  it.each([
    [0, "Aucun projet"],
    [1, "1 projet"],
    [4, "4 projets"],
  ])("%i → %s", (count, label) => {
    expect(projectCountLabel(count)).toBe(label);
  });
});

describe("projectHealth", () => {
  it.each([
    [0, 0, "empty"],
    [3, 3, "healthy"],
    [4, 5, "degraded"],
  ] as const)("%i of %i online → %s", (online, total, health) => {
    expect(projectHealth(online, total)).toBe(health);
  });
});
