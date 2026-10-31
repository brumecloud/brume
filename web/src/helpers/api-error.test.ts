import { describe, expect, it } from "vite-plus/test";

import { apiErrorMessage } from "./api-error";

describe("apiErrorMessage", () => {
  it("uses the message sent by the API", () => {
    expect(apiErrorMessage(409, { message: "Ce nom est déjà pris." })).toBe(
      "Ce nom est déjà pris.",
    );
  });

  it("falls back to the status when the body has no message", () => {
    expect(apiErrorMessage(500, "<html>")).toBe("Erreur 500 du serveur.");
    expect(apiErrorMessage(404, { message: "" })).toBe("Erreur 404 du serveur.");
  });

  it("bounds a very long message", () => {
    expect(apiErrorMessage(400, { message: "x".repeat(1000) })).toHaveLength(300);
  });
});
