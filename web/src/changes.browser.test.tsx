import { describe, expect, it } from "vite-plus/test";
import { userEvent } from "vite-plus/test/browser";

import { brume } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const production = `/projects/${brume.id}/production`;
const service = (name: string) => `${production}/services/${brume.id}-production-${name}`;

// Each test leaves no change staged: the mocked API keeps its state across the tests of a file.
describe("staged changes", () => {
  it("stages a setting with a blue outline, then deploys it", async () => {
    const screen = await renderApp(`${service("brume-api")}/settings`);

    const port = screen.getByRole("textbox", { name: "Port" });
    await port.fill("8080");
    await userEvent.keyboard("{Enter}");

    await expect.element(screen.getByText("Appliquer 1 changement")).toBeVisible();
    expect(port.element().closest("[data-staged]")?.getAttribute("data-staged")).toBe("true");
    await screen.getByRole("button", { name: /^Déployer/ }).click();

    await expect.element(screen.getByText("Appliquer 1 changement")).not.toBeInTheDocument();
    await expect.element(port).toHaveValue("8080");
    expect(port.element().closest("[data-staged]")?.getAttribute("data-staged")).toBe("false");
  });

  it("picks the branch among those of the repository", async () => {
    const screen = await renderApp(`${service("brume-api")}/settings`);

    const branch = screen.getByRole("combobox", { name: "Branche" });
    await expect.element(branch).toHaveTextContent("main");
    await branch.click();
    await screen.getByPlaceholder("Rechercher une branche…").fill("feat");
    await screen.getByRole("option", { name: "feat/canvas" }).click();

    await expect.element(screen.getByText("Appliquer 1 changement")).toBeVisible();
    await screen.getByRole("button", { name: "Autres actions" }).click();
    await screen.getByRole("menuitem", { name: "Annuler tous les changements" }).click();
    await expect.element(branch).toHaveTextContent("main");
  });

  it("deploys on Shift+Enter", async () => {
    const screen = await renderApp(`${service("brume-worker")}/variables`);

    await screen.getByRole("button", { name: "Supprimer CONCURRENCY" }).click();
    await expect.element(screen.getByText("Supprimée")).toBeVisible();
    await userEvent.keyboard("{Shift>}{Enter}{/Shift}");

    await expect.element(screen.getByText("CONCURRENCY")).not.toBeInTheDocument();
    await expect.element(screen.getByText(/^Appliquer/)).not.toBeInTheDocument();
  });

  it("marks the service on the canvas and undoes a change from the details", async () => {
    const screen = await renderApp(`${service("brume-api")}/variables`);

    await screen.getByRole("button", { name: "Supprimer PORT" }).click();
    await expect.element(screen.getByText("1 en attente")).toBeVisible();
    await screen.getByRole("button", { name: "Détails" }).click();
    await expect.element(screen.getByText("PORT supprimée")).toBeVisible();
    await screen.getByRole("button", { name: "Annuler ce changement" }).click();

    await expect.element(screen.getByText(/^Appliquer/)).not.toBeInTheDocument();
    await expect.element(screen.getByText("1 en attente")).not.toBeInTheDocument();
  });

  it("discards every change of the environment", async () => {
    const screen = await renderApp(`${service("brume-api")}/variables`);

    await screen.getByRole("button", { name: "Supprimer PORT" }).click();
    await screen.getByRole("button", { name: "Supprimer R2_ACCOUNT_ID" }).click();
    await expect.element(screen.getByText("Appliquer 2 changements")).toBeVisible();
    await screen.getByRole("button", { name: "Autres actions" }).click();
    await screen.getByRole("menuitem", { name: "Annuler tous les changements" }).click();

    await expect.element(screen.getByText(/^Appliquer/)).not.toBeInTheDocument();
    await expect.element(screen.getByText("Supprimée")).not.toBeInTheDocument();
  });
});

describe("deployment panel", () => {
  it("opens over the service panel and closes back to the list", async () => {
    const screen = await renderApp(`${service("brume-api")}/deployments`);

    await screen.getByRole("article").first().getByRole("link").click();
    const panel = screen.getByRole("region", { name: /^Déploiement/ });
    await expect.element(panel.getByRole("link", { name: "Détails" })).toBeVisible();
    // Secrets stay masked, shared variables keep their origin.
    await expect.element(panel.getByText("••••••••").first()).toBeVisible();
    await expect.element(panel.getByText("Projet").first()).toBeVisible();
    await panel.getByLabelText("Fermer le déploiement").click();

    await expect.element(panel).not.toBeInTheDocument();
    await expect.element(screen.getByText("Historique")).toBeVisible();
  });
});
