import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createMemoryHistory, createRouter } from "@tanstack/react-router";
import { beforeAll, describe, expect, it } from "vite-plus/test";
import { render } from "vitest-browser-react";

import { queryClient } from "~/api/query-client";
import { handlers } from "~/mocks/handlers";
import { brume } from "~/mocks/seed";

import { routeTree } from "./routeTree.gen";

beforeAll(async () => {
  const { network } = await import("virtual:msw");
  network.configure({ handlers, onUnhandledFrame: "bypass" });
  await network.enable();
});

function renderAt(path: string) {
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [path] }),
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

describe("project shell", () => {
  it("opens a project on its production environment", async () => {
    const screen = await renderAt(`/projects/${brume.id}`);

    await expect.element(screen.getByRole("button", { name: "brume" })).toBeVisible();
    await expect.element(screen.getByRole("button", { name: "production" })).toBeVisible();
  });

  it("shows a not-found page for a hostile project id", async () => {
    const screen = await renderAt("/projects/%3Cscript%3E/production");

    await expect.element(screen.getByText("Page introuvable")).toBeVisible();
  });

  it("shows a not-found page for an unknown environment", async () => {
    const screen = await renderAt(`/projects/${brume.id}/preview`);

    await expect.element(screen.getByText("Page introuvable")).toBeVisible();
  });
});
