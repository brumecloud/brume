import { describe, expect, it } from "vite-plus/test";

import { db } from "~/mocks/db";
import { northbean } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const production = `/projects/${northbean.id}/production`;
const service = (name: string) => `${production}/services/${northbean.id}-production-${name}`;

describe("service panel", () => {
  it("opens from the canvas on the deployments of the service", async () => {
    const screen = await renderApp(production);

    await screen.getByText("order-worker").click();

    await expect.element(screen.getByRole("heading", { name: "order-worker" })).toBeVisible();
    await expect.element(screen.getByText("Active")).toBeVisible();
    await expect.element(screen.getByText("History")).toBeVisible();
  });

  it("rolls back to a previous deployment after confirmation", async () => {
    const screen = await renderApp(`${service("shop-api")}/deployments`);

    await screen.getByRole("button", { name: "Deployment actions" }).first().click();
    await screen.getByRole("menuitem", { name: "Roll back to this deployment" }).click();
    await screen.getByRole("button", { name: "Roll back" }).click();

    // The active deployment comes first, above the history.
    const active = screen.getByRole("article").first();
    await expect.element(active.getByText("Active")).toBeVisible();
    await expect
      .element(active.getByText(/^perf\(catalog\): cache the product list/))
      .toBeVisible();
  });

  it("reveals a secret on request only", async () => {
    const screen = await renderApp(`${service("order-worker")}/variables`);

    await expect
      .element(screen.getByText("redis://default:Hn4xT8wz@redis:6379"))
      .not.toBeInTheDocument();
    await screen.getByRole("button", { name: "Reveal REDIS_URL" }).click();

    await expect.element(screen.getByText("redis://default:Hn4xT8wz@redis:6379")).toBeVisible();
  });

  it("warns before a plain variable holds a secret, then adds it", async () => {
    const screen = await renderApp(`${service("order-worker")}/variables`);

    await screen.getByRole("button", { name: "New variable" }).click();
    await screen.getByRole("textbox", { name: "Name" }).fill("STRIPE_KEY");
    await screen.getByRole("textbox", { name: "Value" }).fill("sk_test_123");

    await expect.element(screen.getByText(/looks like a secret/)).toBeVisible();
    await screen.getByRole("button", { name: "Add" }).click();
    await expect.element(screen.getByText("STRIPE_KEY")).toBeVisible();
  });

  it("deletes a service once its name is typed", async () => {
    const screen = await renderApp(`${service("Redis")}/settings`);

    // Deleting a service keeps its volumes, and their data.
    await expect.element(screen.getByText(/Its volume and its data stay/)).toBeVisible();
    await screen.getByRole("button", { name: "Delete service" }).click();
    await expect.element(screen.getByText(/The redis-data volume keeps its data/)).toBeVisible();
    const confirm = screen.getByRole("button", { name: "Delete", exact: true });
    await expect.element(confirm).toBeDisabled();
    await screen.getByRole("textbox", { name: "Service name" }).fill("Redis");
    await confirm.click();

    await expect.element(screen.getByText("redis-data")).not.toBeInTheDocument();
    await expect.element(screen.getByText("order-worker")).toBeVisible();
  });

  it("shows public domains as coming soon", async () => {
    const screen = await renderApp(`${service("shop-api")}/settings`);

    await expect.element(screen.getByRole("textbox", { name: "New domain" })).toBeDisabled();
    await expect
      .element(screen.getByRole("button", { name: "Remove api.northbean.coffee" }))
      .toBeDisabled();
    await expect.element(screen.getByText("Coming soon")).toBeVisible();
  });

  it("refuses a service of another environment", async () => {
    const screen = await renderApp(
      `${production}/services/${northbean.id}-staging-shop-api/deployments`,
    );

    await expect.element(screen.getByText("Page not found")).toBeVisible();
  });

  it("redeploys a service whose deployments all failed", async () => {
    const serviceId = `${northbean.id}-production-order-worker`;
    for (const [index, deployment] of db.deployments.entries()) {
      if (deployment.serviceId === serviceId) {
        db.deployments[index] = { ...deployment, status: "failed" };
      }
    }
    const screen = await renderApp(`${service("order-worker")}/deployments`);

    await expect.element(screen.getByText("No active deployment.")).toBeVisible();
    await screen.getByRole("button", { name: "Redeploy" }).click();

    await expect.element(screen.getByText("No active deployment.")).not.toBeInTheDocument();
    await expect.element(screen.getByRole("article").first().getByText("Active")).toBeVisible();
  });

  it("shows the variables of the revision a rollback puts back", async () => {
    const serviceId = `${northbean.id}-production-shop-api`;
    const screen = await renderApp(`${service("shop-api")}/variables`);
    await expect.element(screen.getByText("CURRENCY")).toBeVisible();
    await screen.getByRole("link", { name: "Deployments", exact: true }).click();

    // The server puts back the variables of the old revision: here, one without CURRENCY.
    db.variables = db.variables.filter(
      (variable) => !(variable.ownerId === serviceId && variable.name === "CURRENCY"),
    );
    await screen.getByRole("button", { name: "Deployment actions" }).first().click();
    await screen.getByRole("menuitem", { name: "Roll back to this deployment" }).click();
    await screen.getByRole("button", { name: "Roll back" }).click();
    await expect.element(screen.getByRole("button", { name: "Roll back" })).not.toBeInTheDocument();
    await screen.getByRole("link", { name: "Variables", exact: true }).click();

    await expect.element(screen.getByText("PORT", { exact: true })).toBeVisible();
    await expect.element(screen.getByText("CURRENCY")).not.toBeInTheDocument();
  });
});

