import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createMemoryHistory, createRouter } from "@tanstack/react-router";
import { beforeAll } from "vite-plus/test";
import { render } from "vitest-browser-react";

import { queryClient } from "~/api/query-client";
import { handlers } from "~/mocks/handlers";
import { routeTree } from "~/routeTree.gen";

// Browser tests go through the same mocked API as the dev server.
beforeAll(async () => {
  const { network } = await import("virtual:msw");
  network.configure({ handlers, onUnhandledFrame: "bypass" });
  await network.enable();
});

// Mounts the whole app, router included, at the given URL.
export function renderApp(path: string) {
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
