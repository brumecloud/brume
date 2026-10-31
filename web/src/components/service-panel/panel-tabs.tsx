import type { ProjectId, ServiceId } from "~/api/types";
import { NavTab, NavTabs } from "~/components/layout/nav-tabs";

interface PanelTabsProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly serviceId: ServiceId;
}

export function PanelTabs({ projectId, environmentName, serviceId }: PanelTabsProps) {
  const params = { projectId, env: environmentName, serviceId };
  return (
    <NavTabs>
      <NavTab to="/projects/$projectId/$env/services/$serviceId/deployments" params={params}>
        Déploiements
      </NavTab>
      <NavTab to="/projects/$projectId/$env/services/$serviceId/variables" params={params}>
        Variables
      </NavTab>
      <NavTab to="/projects/$projectId/$env/services/$serviceId/settings" params={params}>
        Réglages
      </NavTab>
    </NavTabs>
  );
}
