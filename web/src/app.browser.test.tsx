import { describe, expect, it } from "vite-plus/test";

import { brume } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

describe("project shell", () => {
  it("opens a project on its production environment", async () => {
    const screen = await renderApp(`/projects/${brume.id}`);

    await expect
      .element(screen.getByRole("combobox", { name: "Projet" }))
      .toHaveTextContent("brume");
    await expect
      .element(screen.getByRole("combobox", { name: "Environnement" }))
      .toHaveTextContent("production");
  });

  it("searches the environments, then switches", async () => {
    const screen = await renderApp(`/projects/${brume.id}/production`);

    await screen.getByRole("combobox", { name: "Environnement" }).click();
    await screen.getByPlaceholder("Rechercher un environnement…").fill("stag");
    await expect
      .element(screen.getByRole("option", { name: "production" }))
      .not.toBeInTheDocument();
    await screen.getByRole("option", { name: "staging" }).click();

    await expect
      .element(screen.getByRole("combobox", { name: "Environnement" }))
      .toHaveTextContent("staging");
    await expect.element(screen.getByText("Déploiement…")).toBeVisible();
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
