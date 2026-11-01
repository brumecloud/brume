import { afterEach, describe, expect, it } from "vite-plus/test";
import { userEvent } from "vite-plus/test/browser";

import { ingressQuery } from "~/api/ingress";
import { queryClient } from "~/api/query-client";
import { currentSettings, db } from "~/mocks/db";
import { northbean, routelane } from "~/mocks/seed";

import { renderApp } from "../test/render-app";

const settingsOf = (project: { readonly id: string }, name: string) =>
  `/projects/${project.id}/production/services/${project.id}-production-${name}/settings`;
const staged = (element: Element) =>
  element.closest("[data-staged]")?.getAttribute("data-staged") === "true";

// Each test leaves no change staged: the mocked API keeps its state across the tests of a file.
describe("public domain", () => {
  afterEach(() => {
    db.ingress = { zone: "brume.run" };
    queryClient.removeQueries({ queryKey: ingressQuery.queryKey });
  });

  it("stays off until brume install --domain sets up a zone", async () => {
    db.ingress = null;
    queryClient.removeQueries({ queryKey: ingressQuery.queryKey });
    const screen = await renderApp(settingsOf(northbean, "shop-api"));

    await expect.element(screen.getByRole("textbox", { name: "Public domain" })).toBeDisabled();
    await expect.element(screen.getByText("brume install --domain")).toBeVisible();
  });

  it("stages a subdomain of the zone, links to it once deployed, then removes it", async () => {
    const screen = await renderApp(settingsOf(northbean, "shop-api"));

    const domain = screen.getByRole("textbox", { name: "Public domain" });
    await expect.element(screen.getByText(".brume.run")).toBeVisible();
    await domain.fill("Shop");
    await userEvent.keyboard("{Enter}");
    await expect.element(screen.getByText("Apply 1 change")).toBeVisible();
    expect(staged(domain.element())).toBe(true);
    // Staged only: nothing serves it yet.
    await expect
      .element(screen.getByRole("link", { name: "https://shop.brume.run" }))
      .not.toBeInTheDocument();
    await screen.getByRole("button", { name: /^Deploy/ }).click();

    const link = screen.getByRole("link", { name: "https://shop.brume.run" });
    await expect.element(link).toHaveAttribute("href", "https://shop.brume.run/");
    await expect.element(link).toHaveAttribute("target", "_blank");
    await expect.element(domain).toHaveValue("shop");
    await expect.poll(() => staged(domain.element())).toBe(false);

    await domain.clear();
    await userEvent.keyboard("{Enter}");
    await screen.getByRole("button", { name: "Details" }).click();
    await expect.element(screen.getByText("Public domain → none")).toBeVisible();
    await userEvent.keyboard("{Escape}");
    await screen.getByRole("button", { name: /^Deploy/ }).click();
    await expect.element(link).not.toBeInTheDocument();
  });

  it("explains an invalid subdomain or one that another service holds", async () => {
    const holder = db.services.find(({ id }) => id === `${northbean.id}-production-order-worker`);
    if (!holder) {
      throw new Error("order-worker is missing from the seed");
    }
    db.settings.set(holder.id, { ...currentSettings(holder), domain: "orders" });
    try {
      const screen = await renderApp(settingsOf(northbean, "shop-api"));
      const domain = screen.getByRole("textbox", { name: "Public domain" });

      await domain.fill("my_shop");
      await userEvent.keyboard("{Enter}");
      await expect.element(screen.getByText(/^Letters, digits and inner hyphens/)).toBeVisible();
      await expect.element(domain).toHaveAttribute("aria-invalid", "true");

      await domain.fill("orders");
      await userEvent.keyboard("{Enter}");
      await expect
        .element(screen.getByRole("alert"))
        .toHaveTextContent("The subdomain orders is already used by the service order-worker.");
      await expect.element(screen.getByText(/^Apply/)).not.toBeInTheDocument();
    } finally {
      db.settings.delete(holder.id);
    }
  });

  it("shows why a domain without a port does not deploy", async () => {
    const screen = await renderApp(settingsOf(routelane, "sms-gateway"));

    await screen.getByRole("textbox", { name: "Public domain" }).fill("sms");
    await userEvent.keyboard("{Enter}");
    await screen.getByRole("button", { name: /^Deploy/ }).click();
    await expect
      .element(screen.getByRole("alert"))
      .toHaveTextContent("sms-gateway needs a port to have a public domain.");

    await screen.getByRole("button", { name: "More actions" }).click();
    await screen.getByRole("menuitem", { name: "Discard all changes" }).click();
    await expect.element(screen.getByText(/^Apply/)).not.toBeInTheDocument();
  });
});
