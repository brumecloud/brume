import { Outlet, createRootRoute } from "@tanstack/react-router";

import { ErrorPage, NotFoundPage } from "~/components/layout/status-page";
import { TooltipProvider } from "~/components/ui/tooltip";

export const Route = createRootRoute({
  component: RootPage,
  notFoundComponent: NotFoundPage,
  errorComponent: ErrorPage,
});

function RootPage() {
  return (
    <TooltipProvider>
      <Outlet />
    </TooltipProvider>
  );
}
