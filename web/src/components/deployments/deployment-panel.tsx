import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Link } from "@tanstack/react-router";
import { XIcon } from "lucide-react";
import { motion } from "motion/react";
import { Suspense } from "react";

import { deploymentById } from "~/api/deployments";
import { serviceById } from "~/api/services";
import type { DeploymentId, ServiceId } from "~/api/types";
import { ServiceIcon } from "~/components/services/service-icon";
import { NavTab, NavTabs } from "~/components/layout/nav-tabs";
import { StackedPanel } from "~/components/layout/panel-layer";
import { Button } from "~/components/ui/button";
import { Spinner } from "~/components/ui/spinner";
import { type DeploymentSearch, deploymentShortId, formatDateTime } from "~/helpers/deployments";
import { PANEL_TRANSITION } from "~/lib/motion";

import { DeploymentLogs } from "./deployment-logs";
import { DeploymentStatusBadge } from "./deployment-status-badge";
import { RevisionDetails } from "./revision-details";

const FROM = "/projects/$projectId/$env/services/$serviceId/deployments/$deploymentId";

interface DeploymentPanelProps {
  readonly serviceId: ServiceId;
  readonly deploymentId: DeploymentId;
  readonly tab: DeploymentSearch["tab"];
}

// Takes the place of the service panel, which steps back behind it; closing it goes back to the
// list. It slides in opaque: a fade would show the header of the service panel through its own.
export function DeploymentPanel({ serviceId, deploymentId, tab }: DeploymentPanelProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const { data: deployment } = useLiveSuspenseQuery({ query: deploymentById(deploymentId) });
  if (!service || !deployment) {
    return null;
  }

  return (
    <StackedPanel>
      <motion.section
        aria-label={`Déploiement ${deploymentShortId(deployment)}`}
        initial={{ x: 24 }}
        animate={{ x: 0 }}
        transition={PANEL_TRANSITION}
        className="absolute inset-y-2 right-2 left-2 z-20 flex flex-col overflow-hidden rounded-xl border bg-card shadow-2xl md:left-auto md:w-full md:max-w-panel"
      >
        <header className="flex items-center gap-3 px-6 pt-6">
          <ServiceIcon kind={service.kind} />
          <h2 className="truncate font-semibold text-xl">{service.name}</h2>
          <span className="text-muted-foreground">/</span>
          <span className="font-mono text-lg">{deploymentShortId(deployment)}</span>
          <DeploymentStatusBadge status={deployment.status} />
          <span className="ms-auto text-muted-foreground text-sm">
            {formatDateTime(deployment.createdAt)}
          </span>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Fermer le déploiement"
            render={<Link from={FROM} to=".." />}
          >
            <XIcon />
          </Button>
        </header>
        <NavTabs>
          <NavTab from={FROM} to="." search={{}} activeOptions={{ exact: true }}>
            Détails
          </NavTab>
          <NavTab from={FROM} to="." search={{ tab: "build" }} activeOptions={{ exact: true }}>
            Logs de build
          </NavTab>
          <NavTab from={FROM} to="." search={{ tab: "deploy" }} activeOptions={{ exact: true }}>
            Logs d'exécution
          </NavTab>
        </NavTabs>
        <div className="flex min-h-0 flex-1 flex-col overflow-auto">
          <Suspense
            fallback={
              <div className="flex justify-center p-8">
                <Spinner />
              </div>
            }
          >
            {tab === undefined && (
              <div className="p-6">
                <RevisionDetails deployment={deployment} />
              </div>
            )}
            {tab !== undefined && (
              <DeploymentLogs
                deploymentId={deployment.id}
                stream={tab}
                fileName={`${service.slug}-${deploymentShortId(deployment)}-${tab}.log`}
              />
            )}
          </Suspense>
        </div>
      </motion.section>
    </StackedPanel>
  );
}
