import { CSPProvider } from "@base-ui/react/csp-provider";
import { Outlet, createRootRoute } from "@tanstack/react-router";
import { MotionConfig } from "motion/react";

import { ErrorPage, NotFoundPage } from "~/components/layout/status-page";
import { TooltipProvider } from "~/components/ui/tooltip";

export const Route = createRootRoute({
  component: RootPage,
  notFoundComponent: NotFoundPage,
  errorComponent: ErrorPage,
});

// With reduced motion, motion keeps the fades and drops movement and layout animations. The CSP
// of `brume api` refuses inline <style> elements: Base UI's rule lives in styles.css instead.
function RootPage() {
  return (
    <CSPProvider disableStyleElements>
      <MotionConfig reducedMotion="user">
        <TooltipProvider>
          <Outlet />
        </TooltipProvider>
      </MotionConfig>
    </CSPProvider>
  );
}
