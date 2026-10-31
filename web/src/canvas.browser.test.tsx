import { describe, expect, it } from "vite-plus/test";

import { northbean } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

describe("service canvas", () => {
  it("shows the services of the environment with their volumes", async () => {
    const screen = await renderApp(`/projects/${northbean.id}/production`);

    await expect.element(screen.getByText("order-worker")).toBeVisible();
    await expect.element(screen.getByText("api.northbean.coffee")).toBeVisible();
    await expect.element(screen.getByText("postgres-data")).toBeVisible();
    await expect.element(screen.getByText("redis-data")).toBeVisible();
  });

  it("follows the environment of the URL", async () => {
    const screen = await renderApp(`/projects/${northbean.id}/staging`);

    await expect.element(screen.getByText("Deploying…")).toBeVisible();
    await expect.element(screen.getByText("order-worker")).not.toBeInTheDocument();
  });

  it("explains an empty environment", async () => {
    const screen = await renderApp("/projects/new");

    await screen.getByRole("option", { name: /Empty project/ }).click();

    await expect.element(screen.getByText("No services")).toBeVisible();
  });
});
