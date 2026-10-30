import { describe, expect, it } from "vite-plus/test";

import { brume } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const production = `/projects/${brume.id}/production`;
const service = (name: string) => `${production}/services/${brume.id}-production-${name}`;

describe("service panel", () => {
  it("opens from the canvas on the deployments of the service", async () => {
    const screen = await renderApp(production);

    await screen.getByText("brume-worker").click();

    await expect.element(screen.getByRole("heading", { name: "brume-worker" })).toBeVisible();
    await expect.element(screen.getByText("Actif")).toBeVisible();
    await expect.element(screen.getByText("Historique")).toBeVisible();
  });

  it("rolls back to a previous deployment after confirmation", async () => {
    const screen = await renderApp(`${service("brume-api")}/deployments`);

    await screen.getByRole("button", { name: "Actions du déploiement" }).first().click();
    await screen.getByRole("menuitem", { name: "Revenir à ce déploiement" }).click();
    await screen.getByRole("button", { name: "Revenir à ce déploiement" }).click();

    // The active deployment comes first, above the history.
    const active = screen.getByRole("article").first();
    await expect.element(active.getByText("Actif")).toBeVisible();
    await expect.element(active.getByText(/^fix\(api\): converge engine indexes/)).toBeVisible();
  });

  it("reveals a secret on request only", async () => {
    const screen = await renderApp(`${service("brume-worker")}/variables`);

    await expect
      .element(screen.getByText("mongodb://app:Zx91@mongodb:27017/brume"))
      .not.toBeInTheDocument();
    await screen.getByRole("button", { name: "Révéler MONGODB_URI" }).click();

    await expect.element(screen.getByText("mongodb://app:Zx91@mongodb:27017/brume")).toBeVisible();
  });

  it("warns before a plain variable holds a secret, then adds it", async () => {
    const screen = await renderApp(`${service("brume-worker")}/variables`);

    await screen.getByRole("button", { name: "Nouvelle variable" }).click();
    await screen.getByRole("textbox", { name: "Nom" }).fill("STRIPE_KEY");
    await screen.getByRole("textbox", { name: "Valeur" }).fill("sk_test_123");

    await expect.element(screen.getByText(/ressemble à celui d'un secret/)).toBeVisible();
    await screen.getByRole("button", { name: "Ajouter" }).click();
    await expect.element(screen.getByText("STRIPE_KEY")).toBeVisible();
  });

  it("deletes a service once its name is typed", async () => {
    const screen = await renderApp(`${service("Redis")}/settings`);

    await screen.getByRole("button", { name: "Supprimer le service" }).click();
    const confirm = screen.getByRole("button", { name: "Supprimer", exact: true });
    await expect.element(confirm).toBeDisabled();
    await screen.getByRole("textbox", { name: "Nom du service" }).fill("Redis");
    await confirm.click();

    await expect.element(screen.getByText("redis-data")).not.toBeInTheDocument();
    await expect.element(screen.getByText("brume-worker")).toBeVisible();
  });

  it("refuses a service of another environment", async () => {
    const screen = await renderApp(
      `${production}/services/${brume.id}-staging-brume-api/deployments`,
    );

    await expect.element(screen.getByText("Page introuvable")).toBeVisible();
  });
});
