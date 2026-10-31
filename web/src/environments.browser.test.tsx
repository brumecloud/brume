import { describe, expect, it, onTestFinished, vi } from "vite-plus/test";
import { userEvent } from "vite-plus/test/browser";

import { IN_FLIGHT_POLL_MS } from "~/api/services";
import { startContainers } from "~/mocks/db";
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
    await userEvent.keyboard("{Escape}");
    await expect.element(screen.getByRole("dialog")).not.toBeInTheDocument();
  });
});

describe("environment switcher", () => {
  it.each([
    ["shared variables", "variables", "region", "Environment: staging"],
    ["activity", "activity", "heading", "Activity"],
    ["logs", "logs", "searchbox", "Search logs"],
  ] as const)("stays on the %s page", async (_page, path, role, name) => {
    const screen = await renderApp(`/projects/${northbean.id}/production/${path}`);
    const switcher = screen.getByRole("combobox", { name: "Environment" });

    await switcher.click();
    await screen.getByRole("option", { name: "staging" }).click();

    await expect.element(switcher).toHaveTextContent("staging");
    await expect.element(screen.getByRole(role, { name })).toBeVisible();
  });

  it("opens the canvas of the other environment from a service panel", async () => {
    const screen = await renderApp(
      `/projects/${northbean.id}/production/services/${northbean.id}-production-order-worker`,
    );
    const switcher = screen.getByRole("combobox", { name: "Environment" });

    await switcher.click();
    await screen.getByRole("option", { name: "staging" }).click();

    await expect.element(switcher).toHaveTextContent("staging");
    await expect.element(screen.getByText("shop-api", { exact: true })).toBeVisible();
    await expect
      .element(screen.getByRole("heading", { name: "order-worker" }))
      .not.toBeInTheDocument();
  });
});

describe("new service", () => {
  it("adds redis:8 from the canvas, opens its panel and follows it until it runs", async () => {
    // The panel polls while a service deploys: only that interval is simulated, the mocked API
    // still answers in real time. Set before the render, so the poll starts on the fake clock.
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"], shouldClearNativeTimers: true });
    onTestFinished(() => {
      vi.useRealTimers();
    });
    const screen = await renderApp(`/projects/${tallybook.id}/staging`);

    await screen.getByRole("button", { name: "New service" }).click();
    await screen.getByRole("textbox", { name: "Docker image" }).fill("redis:8");
    await screen.getByRole("button", { name: "Add service" }).click();

    await expect.element(screen.getByRole("heading", { name: "redis" })).toBeVisible();
    await expect.element(screen.getByText("Deploying", { exact: true })).toBeVisible();
    startContainers();
    await vi.advanceTimersByTimeAsync(IN_FLIGHT_POLL_MS);
    await expect.element(screen.getByText("Active")).toBeVisible();
    await screen.getByLabelText("Close panel").click();

    // Without the panel, only the card on the canvas carries the name.
    await expect.element(screen.getByText("redis", { exact: true })).toBeVisible();
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
    await userEvent.keyboard("{Escape}");
    await expect.element(screen.getByRole("dialog")).not.toBeInTheDocument();
  });
});
