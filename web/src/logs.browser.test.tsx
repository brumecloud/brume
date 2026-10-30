import { describe, expect, it } from "vite-plus/test";

import { brume } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const production = `/projects/${brume.id}/production`;
const api = `${production}/services/${brume.id}-production-brume-api`;

describe("log explorer", () => {
  it("streams the logs of the environment", async () => {
    const screen = await renderApp(`${production}/logs`);

    await expect.element(screen.getByText(/Background saving started/).first()).toBeVisible();
    await expect.element(screen.getByRole("button", { name: "Pause" })).toBeVisible();
  });

  it("shows a hostile line as text, colors stripped of their escape codes", async () => {
    const screen = await renderApp(`${production}/logs?q=script`);

    await expect.element(screen.getByText(/q=<script>alert\(1\)<\/script>/).first()).toBeVisible();
    expect(document.querySelector("main script")).toBeNull();
    expect(document.querySelector("main")?.textContent).not.toContain("\u001B");
  });

  it("filters on the level from the URL", async () => {
    const screen = await renderApp(`${production}/logs?level=error`);

    await expect.element(screen.getByText(/Redis connection reset/).first()).toBeVisible();
    await expect.element(screen.getByText(/GET \/health/)).not.toBeInTheDocument();
  });
});

describe("deployment details", () => {
  it("shows the revision of a deployment", async () => {
    const screen = await renderApp(`${api}/deployments/${brume.id}-production-brume-api-d0`);

    await expect.element(screen.getByText("Variables de la révision (8)")).toBeVisible();
    await expect.element(screen.getByText("Railpack 0.40.1")).toBeVisible();
  });

  it("shows why a build failed", async () => {
    const screen = await renderApp(
      `${api}/deployments/${brume.id}-production-brume-api-d3?tab=build`,
    );

    await expect.element(screen.getByText(/Build failed/)).toBeVisible();
  });

  it("refuses a deployment of another service", async () => {
    const screen = await renderApp(`${api}/deployments/${brume.id}-production-brume-worker-d0`);

    await expect.element(screen.getByText("Page introuvable")).toBeVisible();
  });
});
