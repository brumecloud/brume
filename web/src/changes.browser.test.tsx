import { describe, expect, it } from "vite-plus/test";
import { userEvent } from "vite-plus/test/browser";

import { northbean } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const production = `/projects/${northbean.id}/production`;
const service = (name: string) => `${production}/services/${northbean.id}-production-${name}`;

// Each test leaves no change staged: the mocked API keeps its state across the tests of a file.
describe("staged changes", () => {
  it("stages a setting with a blue outline, then deploys it", async () => {
    const screen = await renderApp(`${service("shop-api")}/settings`);

    const port = screen.getByRole("textbox", { name: "Port" });
    await port.fill("8080");
    await userEvent.keyboard("{Enter}");

    await expect.element(screen.getByText("Apply 1 change")).toBeVisible();
    expect(port.element().closest("[data-staged]")?.getAttribute("data-staged")).toBe("true");
    await screen.getByRole("button", { name: /^Deploy/ }).click();

    await expect.element(screen.getByText("Apply 1 change")).not.toBeInTheDocument();
    await expect.element(port).toHaveValue("8080");
    expect(port.element().closest("[data-staged]")?.getAttribute("data-staged")).toBe("false");
  });

  it("picks the branch among those of the repository", async () => {
    const screen = await renderApp(`${service("shop-api")}/settings`);

    const branch = screen.getByRole("combobox", { name: "Branch" });
    await expect.element(branch).toHaveTextContent("main");
    await branch.click();
    await screen.getByPlaceholder("Search branches…").fill("feat");
    await screen.getByRole("option", { name: "feat/subscriptions" }).click();

    await expect.element(screen.getByText("Apply 1 change")).toBeVisible();
    await screen.getByRole("button", { name: "More actions" }).click();
    await screen.getByRole("menuitem", { name: "Discard all changes" }).click();
    await expect.element(branch).toHaveTextContent("main");
  });

  it("deploys on Shift+Enter", async () => {
    const screen = await renderApp(`${service("order-worker")}/variables`);

    await screen.getByRole("button", { name: "Delete CONCURRENCY" }).click();
    await expect.element(screen.getByText("Deleted")).toBeVisible();
    await userEvent.keyboard("{Shift>}{Enter}{/Shift}");

    await expect.element(screen.getByText("CONCURRENCY")).not.toBeInTheDocument();
    await expect.element(screen.getByText(/^Apply/)).not.toBeInTheDocument();
  });

  it("marks the service on the canvas and undoes a change from the details", async () => {
    const screen = await renderApp(`${service("shop-api")}/variables`);

    await screen.getByRole("button", { name: "Delete PORT" }).click();
    await expect.element(screen.getByText("1 pending")).toBeVisible();
    await screen.getByRole("button", { name: "Details" }).click();
    await expect.element(screen.getByText("PORT deleted")).toBeVisible();
    await screen.getByRole("button", { name: "Discard this change" }).click();

    await expect.element(screen.getByText(/^Apply/)).not.toBeInTheDocument();
    await expect.element(screen.getByText("1 pending")).not.toBeInTheDocument();
  });

  it("discards every change of the environment", async () => {
    const screen = await renderApp(`${service("shop-api")}/variables`);

    await screen.getByRole("button", { name: "Delete PORT" }).click();
    await screen.getByRole("button", { name: "Delete CURRENCY" }).click();
    await expect.element(screen.getByText("Apply 2 changes")).toBeVisible();
    await screen.getByRole("button", { name: "More actions" }).click();
    await screen.getByRole("menuitem", { name: "Discard all changes" }).click();

    await expect.element(screen.getByText(/^Apply/)).not.toBeInTheDocument();
    await expect.element(screen.getByText("Deleted")).not.toBeInTheDocument();
  });
});

describe("deployment panel", () => {
  it("opens over the service panel and closes back to the list", async () => {
    const screen = await renderApp(`${service("shop-api")}/deployments`);

    await screen.getByRole("article").first().getByRole("link").click();
    const panel = screen.getByRole("region", { name: /^Deployment/ });
    await expect.element(panel.getByRole("link", { name: "Details" })).toBeVisible();
    // Secrets stay masked, shared variables keep their origin.
    await expect.element(panel.getByText("••••••••").first()).toBeVisible();
    await expect.element(panel.getByText("Project").first()).toBeVisible();
    await panel.getByLabelText("Close deployment").click();

    await expect.element(panel).not.toBeInTheDocument();
    await expect.element(screen.getByText("History")).toBeVisible();
  });
});
