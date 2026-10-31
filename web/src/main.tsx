import "./styles.css";

import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { queryClient } from "~/api/query-client";
import { shouldReloadAfterPreloadError } from "~/helpers/reload";

import { routeTree } from "./routeTree.gen";

const router = createRouter({ routeTree, defaultPreload: "intent" });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const RELOAD_KEY = "brume:reloaded-at";

window.addEventListener("vite:preloadError", (event) => {
  const now = Date.now();
  if (shouldReloadAfterPreloadError(sessionStorage.getItem(RELOAD_KEY), now)) {
    event.preventDefault();
    sessionStorage.setItem(RELOAD_KEY, String(now));
    window.location.reload();
  }
});

// Data is mocked until the Rust API exists. Outside dev, this block is dropped from the build.
if (import.meta.env.DEV) {
  const { network } = await import("virtual:msw");
  const { handlers } = await import("./mocks/handlers");
  // Only /api is mocked: pages and Vite modules pass through silently.
  network.configure({ handlers, onUnhandledFrame: "bypass" });
  await network.enable();
}

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </StrictMode>,
  );
}
