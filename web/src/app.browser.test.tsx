import { describe, expect, it } from "vite-plus/test";

import { brume } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

describe("project shell", () => {
  it("opens a project on its production environment", async () => {
    const screen = await renderApp(`/projects/${brume.id}`);

    await expect.element(screen.getByRole("button", { name: "brume" })).toBeVisible();
    await expect.element(screen.getByRole("button", { name: "production" })).toBeVisible();
  });

  it("shows a not-found page for a hostile project id", async () => {
    const screen = await renderApp("/projects/%3Cscript%3E/production");

    await expect.element(screen.getByText("Page introuvable")).toBeVisible();
  });

  it("shows a not-found page for an unknown environment", async () => {
    const screen = await renderApp(`/projects/${brume.id}/preview`);

    await expect.element(screen.getByText("Page introuvable")).toBeVisible();
  });
});
