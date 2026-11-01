import "./styles.css";

import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createHashHistory, createRouter } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { queryClient } from "~/api/query-client";
import { shouldReloadAfterPreloadError } from "~/helpers/reload";

import { routeTree } from "./routeTree.gen";

// The demo embedded on brume.dev (`vp build --mode demo`): the mocked API ships with it, and the
// routes live in the hash, so any static host serves it without rewrites.
const DEMO = import.meta.env.MODE === "demo";

const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  ...(DEMO ? { history: createHashHistory() } : {}),
});

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

// The mocked API: `vp dev --mode mock` and the demo. `vp dev` talks to the real one through the
// Vite proxy. Outside these two modes, this block is dropped from the build.
if (import.meta.env.MODE === "mock" || DEMO) {
  const [{ network }, { handlers }] = await Promise.all([
    import("virtual:msw"),
    import("./mocks/handlers"),
  ]);
  // Only /api is mocked: pages and Vite modules pass through silently.
  network.configure({ handlers, onUnhandledFrame: "bypass" });
  await network.enable();
}

// Tells the landing page that embeds the demo when the first page is on screen, so it can show it.
if (DEMO) {
  const unsubscribe = router.subscribe("onRendered", () => {
    unsubscribe();
    window.parent.postMessage("brume-console:ready", window.location.origin);
  });
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
