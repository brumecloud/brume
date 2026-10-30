import { describe, expect, it } from "vite-plus/test";

import type { EnvironmentId, Service, ServiceId } from "~/api/types";

import { NODE_WIDTH, canvasEdges, canvasKey, canvasNodes, layoutServices } from "./canvas";

const service = (name: string, dependencies: string[] = []): Service => ({
  id: name as ServiceId,
  environmentId: "e-1" as EnvironmentId,
  name,
  slug: name,
  kind: "github",
  status: "online",
  domains: [],
  volume: null,
  dependencies: dependencies as ServiceId[],
});

describe("layoutServices", () => {
  it("puts the services above the ones they depend on", () => {
    const layout = layoutServices([
      service("redis"),
      service("worker", ["mongodb"]),
      service("api", ["redis"]),
      service("mongodb"),
    ]);

    expect(layout.get("api")?.y).toBe(layout.get("worker")?.y);
    expect(layout.get("redis")?.y).toBe(layout.get("mongodb")?.y);
    expect(layout.get("redis")?.y).toBeGreaterThan(layout.get("api")?.y ?? Infinity);
  });

  it("centers each row and orders it by name", () => {
    const layout = layoutServices([service("worker"), service("api")]);
    const half = (NODE_WIDTH + 48) / 2;

    expect(layout.get("api")).toEqual({ x: -half, y: 0 });
    expect(layout.get("worker")).toEqual({ x: half, y: 0 });
  });

  it("stacks a chain of dependencies", () => {
    const layout = layoutServices([service("web", ["api"]), service("api", ["db"]), service("db")]);
    const rows = ["web", "api", "db"].map((id) => layout.get(id)?.y);

    expect(rows).toEqual(rows.toSorted((a = 0, b = 0) => a - b));
    expect(new Set(rows).size).toBe(3);
  });

  it("survives a dependency cycle", () => {
    const layout = layoutServices([service("a", ["b"]), service("b", ["a"])]);

    expect(layout.size).toBe(2);
  });
});

describe("canvasNodes", () => {
  const services = [service("api", ["redis"]), service("redis")];

  it("places each service from the layout", () => {
    const nodes = canvasNodes(services, []);

    expect(nodes.map((node) => [node.id, node.type])).toEqual([
      ["api", "service"],
      ["redis", "service"],
    ]);
    expect(nodes[0]?.position).toEqual(layoutServices(services).get("api"));
  });

  it("prefers the position the admin saved", () => {
    const nodes = canvasNodes(services, [{ id: "api", x: 10, y: 20 }]);

    expect(nodes[0]?.position).toEqual({ x: 10, y: 20 });
  });

  it("ignores a corrupted saved position", () => {
    const nodes = canvasNodes(services, [{ id: "api", x: Number.NaN, y: 20 }]);

    expect(nodes[0]?.position).toEqual(layoutServices(services).get("api"));
  });
});

describe("canvasEdges", () => {
  it("links each service to its dependencies in the environment", () => {
    const edges = canvasEdges([service("api", ["redis", "elsewhere"]), service("redis")]);

    expect(edges).toEqual([
      expect.objectContaining({ id: "api->redis", source: "api", target: "redis" }),
    ]);
  });
});

describe("canvasKey", () => {
  it("does not depend on the order of the services", () => {
    expect(canvasKey([service("a"), service("b", ["a"])])).toBe(
      canvasKey([service("b", ["a"]), service("a")]),
    );
  });

  it("changes when a service or a dependency appears", () => {
    const key = canvasKey([service("a"), service("b")]);

    expect(canvasKey([service("a"), service("b"), service("c")])).not.toBe(key);
    expect(canvasKey([service("a"), service("b", ["a"])])).not.toBe(key);
  });
});
