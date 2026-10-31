import { describe, expect, it } from "vite-plus/test";

import { environmentNameError } from "./environments";

describe("environmentNameError", () => {
  it("accepts a DNS label", () => {
    expect(environmentNameError("staging", ["production"])).toBeNull();
  });

  it("refuses uppercase letters", () => {
    expect(environmentNameError("Staging", [])).toEqual(expect.any(String));
  });

  it("refuses a leading hyphen", () => {
    expect(environmentNameError("-x", [])).toEqual(expect.any(String));
  });

  it("refuses an empty name", () => {
    expect(environmentNameError("", [])).toEqual(expect.any(String));
  });

  it("refuses a name longer than a DNS label", () => {
    expect(environmentNameError("a".repeat(64), [])).toEqual(expect.any(String));
  });

  it("refuses a name already taken in the project", () => {
    expect(environmentNameError("staging", ["production", "staging"])).toBe(
      "An environment named staging already exists.",
    );
  });
});
