import { describe, expect, it } from "vite-plus/test";

import { renderApp } from "../test/render-app";

describe("project dashboard", () => {
  it("shows each project with the health of its default environment", async () => {
    const screen = await renderApp("/projects");

    await expect.element(screen.getByText("4 projets")).toBeVisible();
    await expect
      .element(screen.getByRole("link", { name: /eve.*dev.*4\/5 services en ligne/ }))
      .toBeVisible();
  });

  it("filters the projects from the search in the URL", async () => {
    const screen = await renderApp("/projects?q=redis");

    await expect.element(screen.getByText("1 projet")).toBeVisible();
    await expect.element(screen.getByRole("link", { name: /^brume/ })).toBeVisible();
  });

  it("explains an empty search", async () => {
    const screen = await renderApp("/projects?q=nothing-matches");

    await expect
      .element(screen.getByText("Aucun projet ne correspond à cette recherche."))
      .toBeVisible();
  });
});

describe("new project", () => {
  it("creates a project from a database template and opens it", async () => {
    const screen = await renderApp("/projects/new?step=database");

    await screen.getByRole("option", { name: /Redis/ }).click();

    await expect.element(screen.getByRole("button", { name: /^redis/ })).toBeVisible();
    await expect.element(screen.getByRole("button", { name: "production" })).toBeVisible();
  });

  it("refuses an invalid image reference", async () => {
    const screen = await renderApp("/projects/new?step=image");

    await screen.getByRole("textbox", { name: "Image Docker" }).fill("Not An Image");
    await screen.getByRole("button", { name: "Déployer l'image" }).click();

    await expect.element(screen.getByText(/Référence d'image invalide/)).toBeVisible();
  });
});
