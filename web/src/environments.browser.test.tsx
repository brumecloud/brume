import { describe, expect, it } from "vite-plus/test";

import { northbean, routelane, tallybook } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

describe("new environment", () => {
  it("creates staging, opens it and lists it in the switcher", async () => {
    const screen = await renderApp(`/projects/${routelane.id}/production`);

    await screen.getByRole("combobox", { name: "Environment" }).click();
    await screen.getByRole("button", { name: "New environment" }).click();
    await screen.getByRole("textbox", { name: "Name" }).fill("staging");
    await screen.getByRole("button", { name: "Create" }).click();

    const switcher = screen.getByRole("combobox", { name: "Environment" });
    await expect.element(switcher).toHaveTextContent("staging");
    await expect.element(screen.getByText("No services")).toBeVisible();
    await switcher.click();
    await expect.element(screen.getByRole("option", { name: "production" })).toBeVisible();
    await expect.element(screen.getByRole("option", { name: "staging" })).toBeVisible();
  });

  it("refuses a name the project already uses", async () => {
    const screen = await renderApp(`/projects/${northbean.id}/production`);

    await screen.getByRole("combobox", { name: "Environment" }).click();
    await screen.getByRole("button", { name: "New environment" }).click();
    await screen.getByRole("textbox", { name: "Name" }).fill("staging");
    await screen.getByRole("button", { name: "Create" }).click();

    await expect
      .element(screen.getByText("An environment named staging already exists."))
      .toBeVisible();
  });
});

describe("new service", () => {
  it("adds redis:8 from the canvas and opens its panel", async () => {
    const screen = await renderApp(`/projects/${tallybook.id}/staging`);

    await screen.getByRole("button", { name: "New service" }).click();
    await screen.getByRole("textbox", { name: "Docker image" }).fill("redis:8");
    await screen.getByRole("button", { name: "Add service" }).click();

    await expect.element(screen.getByRole("heading", { name: "redis" })).toBeVisible();
    await expect.element(screen.getByText("redis", { exact: true }).first()).toBeVisible();
    await expect.element(screen.getByText("Active")).toBeVisible();
  });

  it("adds a service to an empty environment", async () => {
    const screen = await renderApp("/projects/new");

    await screen.getByRole("option", { name: /Empty project/ }).click();
    await screen.getByRole("button", { name: "New service" }).click();
    await screen.getByRole("textbox", { name: "Docker image" }).fill("nginx:1.29");
    await screen.getByRole("textbox", { name: "Name" }).fill("web");
    await screen.getByRole("button", { name: "Add service" }).click();

    await expect.element(screen.getByRole("heading", { name: "web" })).toBeVisible();
  });

  it("refuses an invalid image reference", async () => {
    const screen = await renderApp(`/projects/${tallybook.id}/staging`);

    await screen.getByRole("button", { name: "New service" }).click();
    await screen.getByRole("textbox", { name: "Docker image" }).fill("Not An Image");
    await screen.getByRole("button", { name: "Add service" }).click();

    await expect.element(screen.getByText(/Invalid image reference/)).toBeVisible();
  });
});
