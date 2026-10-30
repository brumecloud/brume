import { describe, expect, it } from "vite-plus/test";

import { DecodeError, decodeArray, decodeRecord, readOneOf, readString } from "./decode";

describe("decodeRecord", () => {
  it("accepts a plain object", () => {
    expect(decodeRecord({ a: 1 }, "project")).toEqual({ a: 1 });
  });

  it.each([null, [], "x", 3])("rejects %j", (raw) => {
    expect(() => decodeRecord(raw, "project")).toThrow(DecodeError);
  });
});

describe("decodeArray", () => {
  it("decodes each item", () => {
    expect(decodeArray([1, 2], (item) => Number(item) * 2, "list")).toEqual([2, 4]);
  });

  it("rejects a non-array", () => {
    expect(() => decodeArray({}, (item) => item, "list")).toThrow("list: expected an array");
  });
});

describe("readString", () => {
  it("reads a string field", () => {
    expect(readString({ name: "brume" }, "name", "project")).toBe("brume");
  });

  it("rejects a missing or non-string field", () => {
    expect(() => readString({ name: 3 }, "name", "project")).toThrow(
      "project.name: expected a string",
    );
    expect(() => readString({}, "name", "project")).toThrow(DecodeError);
  });
});

describe("readOneOf", () => {
  const kinds = ["github", "image"] as const;

  it("reads one of the allowed values", () => {
    expect(readOneOf({ kind: "image" }, "kind", kinds, "service")).toBe("image");
  });

  it("rejects any other value", () => {
    expect(() => readOneOf({ kind: "lambda" }, "kind", kinds, "service")).toThrow(
      "service.kind: expected one of github, image",
    );
  });
});
