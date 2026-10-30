import { Link } from "@tanstack/react-router";

import type { ProjectId, ServiceId } from "~/api/types";

interface PanelTabsProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly serviceId: ServiceId;
}

const TAB_CLASSES =
  "border-transparent border-b-2 px-1 pb-3 text-muted-foreground text-sm transition-colors hover:text-foreground data-[status=active]:border-foreground data-[status=active]:text-foreground";

// Each tab is a URL: links, not ARIA tabs.
export function PanelTabs({ projectId, environmentName, serviceId }: PanelTabsProps) {
  const params = { projectId, env: environmentName, serviceId };
  return (
    <nav className="mt-4 flex gap-6 border-b px-6">
      <Link
        to="/projects/$projectId/$env/services/$serviceId/deployments"
        params={params}
        className={TAB_CLASSES}
      >
        Déploiements
      </Link>
      <Link
        to="/projects/$projectId/$env/services/$serviceId/variables"
        params={params}
        className={TAB_CLASSES}
      >
        Variables
      </Link>
      <Link
        to="/projects/$projectId/$env/services/$serviceId/settings"
        params={params}
        className={TAB_CLASSES}
      >
        Réglages
      </Link>
    </nav>
  );
}
