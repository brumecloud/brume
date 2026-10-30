import { describe, expect, it } from "vite-plus/test";

import { brume } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

describe("service canvas", () => {
  it("shows the services of the environment with their volumes", async () => {
    const screen = await renderApp(`/projects/${brume.id}/production`);

    await expect.element(screen.getByText("brume-worker")).toBeVisible();
    await expect.element(screen.getByText("api.brume.dev")).toBeVisible();
    await expect.element(screen.getByText("mongodb-data")).toBeVisible();
    await expect.element(screen.getByText("redis-data")).toBeVisible();
  });

  it("follows the environment of the URL", async () => {
    const screen = await renderApp(`/projects/${brume.id}/staging`);

    await expect.element(screen.getByText("Déploiement…")).toBeVisible();
    await expect.element(screen.getByText("brume-worker")).not.toBeInTheDocument();
  });

  it("explains an empty environment", async () => {
    const screen = await renderApp("/projects/new");

    await screen.getByRole("option", { name: /Projet vide/ }).click();

    await expect.element(screen.getByText("Aucun service")).toBeVisible();
  });
});
