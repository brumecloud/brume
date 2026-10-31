import { describe, expect, it } from "vite-plus/test";

import { northbean } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

describe("project shell", () => {
  it("opens a project on its production environment", async () => {
    const screen = await renderApp(`/projects/${northbean.id}`);

    await expect
      .element(screen.getByRole("combobox", { name: "Project" }))
      .toHaveTextContent("northbean");
    await expect
      .element(screen.getByRole("combobox", { name: "Environment" }))
      .toHaveTextContent("production");
  });

  it("searches the environments, then switches", async () => {
    const screen = await renderApp(`/projects/${northbean.id}/production`);

    await screen.getByRole("combobox", { name: "Environment" }).click();
    await screen.getByPlaceholder("Search environments…").fill("stag");
    await expect
      .element(screen.getByRole("option", { name: "production" }))
      .not.toBeInTheDocument();
    await screen.getByRole("option", { name: "staging" }).click();

    await expect
      .element(screen.getByRole("combobox", { name: "Environment" }))
      .toHaveTextContent("staging");
    await expect.element(screen.getByText("Deploying…")).toBeVisible();
  });

  it("shows a not-found page for a hostile project id", async () => {
    const screen = await renderApp("/projects/%3Cscript%3E/production");

    await expect.element(screen.getByText("Page not found")).toBeVisible();
  });

  it("shows a not-found page for an unknown environment", async () => {
    const screen = await renderApp(`/projects/${northbean.id}/preview`);

    await expect.element(screen.getByText("Page not found")).toBeVisible();
  });
});
