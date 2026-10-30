import { describe, expect, it } from "vite-plus/test";

import type { Deployment, DeploymentId, DeploymentStatus, ServiceId } from "~/api/types";

import { DecodeError } from "./decode";
import {
  deploymentTitle,
  formatDateTime,
  parseDeploymentId,
  parseDeploymentSearch,
  parseDeployments,
  parseRevision,
  splitDeployments,
} from "./deployments";

const deployment = (
  id: string,
  status: DeploymentStatus,
  createdAt: string,
  overrides: Partial<Deployment> = {},
): Deployment => ({
  id: id as DeploymentId,
  serviceId: "s-1" as ServiceId,
  status,
  createdAt,
  author: "GitHub",
  commit: null,
  image: null,
  rollbackable: true,
  ...overrides,
});

describe("parseDeployments", () => {
  const raw = {
    id: "d-1",
    serviceId: "s-1",
    status: "active",
    createdAt: "2026-10-11T20:53:00Z",
    author: "GitHub",
    commit: { sha: "d63eac96f1", message: "feat: x", branch: "main" },
    image: null,
    rollbackable: true,
  };

  it("decodes the API response", () => {
    expect(parseDeployments([raw])).toEqual([raw]);
  });

  it("rejects a malformed commit or status", () => {
    expect(() => parseDeployments([{ ...raw, commit: { sha: "x" } }])).toThrow(DecodeError);
    expect(() => parseDeployments([{ ...raw, status: "skipped" }])).toThrow(DecodeError);
  });
});

describe("splitDeployments", () => {
  it("puts the active deployment apart and the rest newest first", () => {
    const old = deployment("d-1", "superseded", "2026-10-01T10:00:00Z");
    const active = deployment("d-2", "active", "2026-10-02T10:00:00Z");
    const failed = deployment("d-3", "failed", "2026-10-03T10:00:00Z");

    expect(splitDeployments([old, active, failed])).toEqual({
      active,
      others: [failed, old],
    });
  });

  it("handles a service that never deployed", () => {
    expect(splitDeployments([])).toEqual({ active: undefined, others: [] });
  });
});

describe("deploymentTitle", () => {
  it("uses the first line of the commit message", () => {
    const commit = { sha: "abc", message: "fix(api): converge\n\nlong body", branch: "main" };

    expect(deploymentTitle(deployment("d", "active", "", { commit }))).toBe("fix(api): converge");
  });

  it("falls back to the image, then to a generic title", () => {
    expect(deploymentTitle(deployment("d", "active", "", { image: "redis:7.4" }))).toBe(
      "redis:7.4",
    );
    expect(deploymentTitle(deployment("d", "active", ""))).toBe("Déploiement");
  });
});

describe("formatDateTime", () => {
  it("formats a date and an hour in French", () => {
    expect(formatDateTime("2026-10-11T12:53:00Z")).toMatch(/^11 oct\. 2026, \d{2}:53$/);
  });
});

describe("parseRevision", () => {
  const raw = {
    digest: "sha256:4f1c",
    builder: "Railpack 0.40.1",
    startCommand: "pnpm start",
    variables: [
      { name: "PORT", kind: "plain", value: "3000", origin: "service" },
      { name: "REDIS_URL", kind: "secret", value: null, origin: "service" },
    ],
  };

  it("decodes the revision of a deployment", () => {
    expect(parseRevision(raw)).toEqual(raw);
  });

  it("rejects a variable without origin", () => {
    expect(() =>
      parseRevision({ ...raw, variables: [{ name: "PORT", kind: "plain", value: "1" }] }),
    ).toThrow(DecodeError);
  });
});

describe("parseDeploymentSearch", () => {
  it("keeps the build and deploy log tabs only", () => {
    expect(parseDeploymentSearch({ tab: "build" })).toEqual({ tab: "build" });
    expect(parseDeploymentSearch({ tab: "deploy" })).toEqual({ tab: "deploy" });
    expect(parseDeploymentSearch({ tab: "network" })).toEqual({});
  });
});

describe("parseDeploymentId", () => {
  it("accepts an id and rejects anything else", () => {
    expect(parseDeploymentId("d-1")).toBe("d-1");
    expect(parseDeploymentId("<x>")).toBeUndefined();
  });
});
