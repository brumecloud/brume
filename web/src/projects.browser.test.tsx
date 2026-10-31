import { describe, expect, it } from "vite-plus/test";

import { renderApp } from "../test/render-app";

describe("project dashboard", () => {
  it("shows each project with the health of its default environment", async () => {
    const screen = await renderApp("/projects");

    await expect.element(screen.getByText("5 projects")).toBeVisible();
    await expect
      .element(screen.getByRole("link", { name: /routelane.*production.*6\/7 services online/ }))
      .toBeVisible();
  });

  it("filters the projects from the search in the URL", async () => {
    const screen = await renderApp("/projects?q=invoices");

    await expect.element(screen.getByText("1 project")).toBeVisible();
    await expect.element(screen.getByRole("link", { name: /^tallybook/ })).toBeVisible();
  });

  it("explains an empty search", async () => {
    const screen = await renderApp("/projects?q=nothing-matches");

    await expect.element(screen.getByText("No projects match this search.")).toBeVisible();
  });
});

describe("new project", () => {
  it("creates a project from a public image and opens it", async () => {
    const screen = await renderApp("/projects/new?step=image");

    await screen.getByRole("textbox", { name: "Docker image" }).fill("redis:8");
    await screen.getByRole("button", { name: "Deploy image" }).click();

    await expect
      .element(screen.getByRole("combobox", { name: "Project" }))
      .toHaveTextContent("redis");
    await expect
      .element(screen.getByRole("combobox", { name: "Environment" }))
      .toHaveTextContent("production");
  });

  it.each(["GitHub repository", "Database", "Static site"])(
    "shows %s as coming soon, not as a choice",
    async (name) => {
      const screen = await renderApp("/projects/new");

      const option = screen.getByRole("option", { name: new RegExp(name) });
      await expect.element(option).toHaveAttribute("aria-disabled", "true");
      await expect.element(option.getByText("Coming soon")).toBeVisible();
    },
  );

  it("offers a Docker image", async () => {
    const screen = await renderApp("/projects/new");

    await screen.getByRole("option", { name: /Docker image/ }).click();

    await expect.element(screen.getByRole("textbox", { name: "Docker image" })).toBeVisible();
  });

  it("says when the server cannot build a source yet", async () => {
    const screen = await renderApp("/projects/new?step=database");

    await screen.getByRole("option", { name: /Redis/ }).click();

    await expect.element(screen.getByText("redis sources are not available yet.")).toBeVisible();
  });

  it("refuses an invalid image reference", async () => {
    const screen = await renderApp("/projects/new?step=image");

    await screen.getByRole("textbox", { name: "Docker image" }).fill("Not An Image");
    await screen.getByRole("button", { name: "Deploy image" }).click();

    await expect.element(screen.getByText(/Invalid image reference/)).toBeVisible();
  });
});
