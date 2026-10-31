import { describe, expect, it } from "vite-plus/test";

import { db } from "~/mocks/db";
import { northbean, tallybook } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const sharedPage = (projectId: string) => `/projects/${projectId}/production/variables`;
const deploymentCount = () => db.deployments.length;

// Each test leaves no change staged: the mocked API keeps its state across the tests of a file.
describe("shared variables", () => {
  it("shows the project's and the environment's variables", async () => {
    const screen = await renderApp(sharedPage(northbean.id));

    const project = screen.getByRole("region", { name: "Project" });
    const environment = screen.getByRole("region", { name: "Environment: production" });
    await expect.element(project.getByText("LOG_LEVEL")).toBeVisible();
    await expect.element(environment.getByText("APP_URL")).toBeVisible();
  });

  it("lists the services a project variable redeploys, then deploys them", async () => {
    const screen = await renderApp(sharedPage(tallybook.id));
    const before = deploymentCount();

    await screen.getByRole("button", { name: "Edit TZ" }).click();
    await screen.getByRole("textbox", { name: "Value" }).fill("UTC");
    await screen.getByRole("button", { name: "Save" }).click();

    const impact = screen.getByRole("alertdialog");
    await expect.element(impact.getByText("This change redeploys 3 services.")).toBeVisible();
    await expect.element(impact.getByText("production")).toBeVisible();
    await expect.element(impact.getByText("billing-api, pdf-renderer")).toBeVisible();
    await expect.element(impact.getByText("staging")).toBeVisible();
    await impact.getByRole("button", { name: "Redeploy 3 services" }).click();

    await expect.element(screen.getByText("UTC")).toBeVisible();
    expect(deploymentCount()).toBe(before + 3);
  });

  it("saves a shared secret at once and offers to redeploy what references it", async () => {
    const screen = await renderApp(sharedPage(northbean.id));
    const before = deploymentCount();

    await screen.getByRole("button", { name: "Edit SENTRY_DSN" }).click();
    await screen.getByLabelText("Value").fill("https://new@o4512.ingest.sentry.io/1042");
    await screen.getByRole("button", { name: "Save" }).click();

    const prompt = screen.getByRole("alertdialog");
    await expect.element(prompt.getByText(/keep the old value/)).toBeVisible();
    await expect.element(prompt.getByText("shop-api").first()).toBeVisible();
    expect(deploymentCount()).toBe(before);
    await prompt.getByRole("button", { name: "Redeploy now" }).click();

    await expect.element(prompt).not.toBeInTheDocument();
    expect(deploymentCount()).toBeGreaterThan(before);
  });

  it("reminds that a project secret also reaches staging", async () => {
    const screen = await renderApp(sharedPage(northbean.id));

    await screen
      .getByRole("region", { name: "Project" })
      .getByRole("button", { name: "New variable" })
      .click();
    await screen.getByRole("combobox", { name: "Type" }).click();
    await screen.getByRole("option", { name: "Secret" }).click();

    await expect
      .element(
        screen.getByText(
          "Project secrets also reach staging. Consider defining it per environment.",
        ),
      )
      .toBeVisible();
  });

  it("shows why a secret still in use cannot be deleted", async () => {
    const screen = await renderApp(sharedPage(northbean.id));

    await screen.getByRole("button", { name: "Delete SENTRY_DSN" }).click();
    await screen.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click();

    await expect.element(screen.getByText(/SENTRY_DSN is still referenced by/)).toBeVisible();
  });
});

describe("secret references", () => {
  it("references a shared secret from a service: a staged change without a value", async () => {
    const screen = await renderApp(
      `/projects/${northbean.id}/production/services/${northbean.id}-production-order-worker/variables`,
    );

    await screen.getByRole("combobox", { name: "Reference a shared secret" }).click();
    await screen.getByRole("option", { name: "SENTRY_DSN" }).click();

    await expect.element(screen.getByText("Apply 1 change")).toBeVisible();
    const staged = db.changes.find(
      (change) => change.type === "variable" && change.name === "SENTRY_DSN",
    );
    expect(staged).toMatchObject({ kind: "secret", value: null });
    expect(staged && db.stagedSecrets.has(staged.id)).toBe(false);

    await screen.getByRole("button", { name: /^Deploy/ }).click();
    await expect.element(screen.getByText(/^Apply/)).not.toBeInTheDocument();
    await expect.element(screen.getByText("Shared")).toBeVisible();
  });
});
