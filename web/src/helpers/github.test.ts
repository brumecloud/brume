import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import { parseBranches, repositoryPath } from "./github";

describe("repositoryPath", () => {
  it("keeps a GitHub repository name", () => {
    expect(repositoryPath("brumecloud/brume")).toBe("brumecloud/brume");
    expect(repositoryPath("planchon/ai-agent.v2")).toBe("planchon/ai-agent.v2");
  });

  it.each(["brume", "a/b/c", "a/..", "../b", "a/b?x=1", "a/%2e%2e"])("rejects %j", (name) => {
    expect(repositoryPath(name)).toBeUndefined();
  });
});

describe("parseBranches", () => {
  it("decodes a list of branch names", () => {
    expect(parseBranches(["main", "feat/logs"])).toEqual(["main", "feat/logs"]);
  });

  it("rejects anything else", () => {
    expect(() => parseBranches([1])).toThrow(DecodeError);
    expect(() => parseBranches("main")).toThrow(DecodeError);
  });
});
