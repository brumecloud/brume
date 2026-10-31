import { describe, expect, it } from "vite-plus/test";

import { parseLoginHash, redirectsToLogin } from "./session";

describe("parseLoginHash", () => {
  it("reads the token of a login link", () => {
    expect(parseLoginHash("#token=abc_DEF-123456789012")).toBe("abc_DEF-123456789012");
  });

  it("refuses an empty token", () => {
    expect(parseLoginHash("#token=")).toBeNull();
  });

  it("refuses a hash without a token", () => {
    expect(parseLoginHash("#x=1")).toBeNull();
  });

  it("refuses a token too long to be one", () => {
    expect(parseLoginHash(`#token=${"a".repeat(200)}`)).toBeNull();
  });

  it("refuses a token too short to be one", () => {
    expect(parseLoginHash("#token=abc")).toBeNull();
  });

  it("refuses characters outside base64url", () => {
    expect(parseLoginHash("#token=abc+DEF/123456789012")).toBeNull();
  });

  it("reads a hash without its leading #, as the router gives it", () => {
    expect(parseLoginHash("token=abc_DEF-123456789012")).toBe("abc_DEF-123456789012");
  });
});

describe("redirectsToLogin", () => {
  it("sends an expired session to the login page", () => {
    expect(redirectsToLogin(401, "/api/projects")).toBe(true);
  });

  it("lets the session routes report a 401 themselves", () => {
    expect(redirectsToLogin(401, "/api/session")).toBe(false);
  });

  it("ignores the other errors", () => {
    expect(redirectsToLogin(403, "/api/projects")).toBe(false);
  });
});
