import { HttpResponse, http } from "msw";
import { network } from "virtual:msw";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { page } from "vite-plus/test/browser";

import { northbean } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const event = (seq: number, summary: string) => ({
  seq,
  at: "2026-10-31T10:00:00.000Z",
  type: "ProjectCreated",
  actor: "admin",
  environmentId: null,
  serviceId: null,
  deploymentId: null,
  summary,
});

describe("activity", () => {
  afterEach(() => network.resetHandlers());

  it("lists ProjectCreated first for a new project", async () => {
    // The rail only shows from the md breakpoint.
    await page.viewport(1280, 800);
    const screen = await renderApp("/projects/new");

    await screen.getByRole("option", { name: /Empty project/ }).click();
    await expect.element(screen.getByText("No services")).toBeVisible();
    await screen.getByRole("link", { name: "Activity" }).click();

    const first = screen.getByRole("listitem").first();
    await expect.element(first.getByText("ProjectCreated")).toBeVisible();
    await expect.element(first.getByText("admin")).toBeVisible();
  });

  it("shows the history of a seeded project, deployments included", async () => {
    const screen = await renderApp(`/projects/${northbean.id}/production/activity`);

    await expect.element(screen.getByText("DeploymentSucceeded").first()).toBeVisible();
  });

  it("records a reveal without the value", async () => {
    await page.viewport(1280, 800);
    const service = `${northbean.id}-production-order-worker`;
    const screen = await renderApp(
      `/projects/${northbean.id}/production/services/${service}/variables`,
    );
    await screen.getByRole("button", { name: "Reveal REDIS_URL" }).click();
    await expect.element(screen.getByText("redis://default:Hn4xT8wz@redis:6379")).toBeVisible();
    // A second render in the same test would blank the next tests: navigate instead.
    await screen.getByRole("link", { name: "Activity" }).click();

    const last = screen.getByRole("listitem").last();
    await expect.element(last.getByText("SecretRevealed")).toBeVisible();
    await expect.element(last.getByText("Revealed REDIS_URL")).toBeVisible();
    expect(document.body.textContent).not.toContain("Hn4xT8wz");
  });

  it("renders a summary as text, never as HTML", async () => {
    network.use(
      http.get("/api/events", () =>
        HttpResponse.json({
          events: [event(1, '<img src=x onerror="alert(1)">')],
          nextBefore: null,
        }),
      ),
    );
    const screen = await renderApp(`/projects/${northbean.id}/production/activity`);

    await expect.element(screen.getByText('<img src=x onerror="alert(1)">')).toBeVisible();
    expect(document.querySelector('img[src="x"]')).toBeNull();
  });

  it("loads earlier events above the ones on screen", async () => {
    network.use(
      http.get("/api/events", ({ request }) => {
        const before = new URL(request.url).searchParams.get("before");
        return HttpResponse.json(
          before === "150"
            ? { events: [event(10, "Older event")], nextBefore: null }
            : { events: [event(150, "Newer event")], nextBefore: 150 },
        );
      }),
    );
    const screen = await renderApp(`/projects/${northbean.id}/production/activity`);

    await screen.getByRole("button", { name: "Load earlier events" }).click();

    await expect
      .element(screen.getByRole("listitem").first().getByText("Older event"))
      .toBeVisible();
    await expect
      .element(screen.getByRole("listitem").last().getByText("Newer event"))
      .toBeVisible();
    await expect
      .element(screen.getByRole("button", { name: "Load earlier events" }))
      .not.toBeInTheDocument();
  });
});
