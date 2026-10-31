import { describe, expect, it } from "vite-plus/test";

import { apiErrorMessage } from "./api-error";

describe("apiErrorMessage", () => {
  it("uses the message sent by the API", () => {
    expect(apiErrorMessage(409, { message: "This name is already taken." })).toBe(
      "This name is already taken.",
    );
  });

  it("falls back to the status when the body has no message", () => {
    expect(apiErrorMessage(500, "<html>")).toBe("Server error 500.");
    expect(apiErrorMessage(404, { message: "" })).toBe("Server error 404.");
  });

  it("bounds a very long message", () => {
    expect(apiErrorMessage(400, { message: "x".repeat(1000) })).toHaveLength(300);
  });
});
