import { HttpResponse, http } from "msw";
import { network } from "virtual:msw";
import { afterEach, describe, expect, it } from "vite-plus/test";

import { renderApp } from "../test/render-app";

const unauthorized = () => HttpResponse.json({ message: "Not signed in." }, { status: 401 });

describe("login", () => {
  afterEach(() => network.resetHandlers());

  it("signs in from the link of brume dev and opens the projects", async () => {
    const screen = await renderApp("/login#token=abc_DEF-123456789012");

    await expect.element(screen.getByText("5 projects")).toBeVisible();
  });

  it("explains where the login link comes from", async () => {
    network.use(http.get("/api/session", unauthorized));
    const screen = await renderApp("/login");

    await expect.element(screen.getByText(/Open the login link that/)).toBeVisible();
    await expect.element(screen.getByText("sudo brume admin login")).toBeVisible();
    await expect.element(screen.getByText("brume dev")).toBeVisible();
  });

  it("says when a login link no longer works", async () => {
    network.use(http.post("/api/session", unauthorized));
    const screen = await renderApp("/login#token=abc_DEF-123456789012");

    await expect.element(screen.getByText(/link has expired or was already used/)).toBeVisible();
  });
});
