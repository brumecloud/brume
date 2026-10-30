import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Link } from "@tanstack/react-router";
import { XIcon } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import type { ReactNode } from "react";

import { serviceById } from "~/api/services";
import type { ProjectId, ServiceId } from "~/api/types";
import { ServiceIcon } from "~/components/services/service-icon";
import { Button } from "~/components/ui/button";
import { PANEL_TRANSITION } from "~/lib/motion";

import { PanelTabs } from "./panel-tabs";

interface ServicePanelProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly serviceId: ServiceId;
  readonly children: ReactNode;
}

// Reference: infra/screens/04a-service-deployments.png. The panel slides in from the right,
// where it comes from; with reduced motion it only fades in.
export function ServicePanel({
  projectId,
  environmentName,
  serviceId,
  children,
}: ServicePanelProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const reduceMotion = useReducedMotion();
  if (!service) {
    return null;
  }

  return (
    <motion.aside
      aria-label={service.name}
      initial={{ opacity: 0, transform: reduceMotion ? "none" : "translateX(24px)" }}
      animate={{ opacity: 1, transform: "translateX(0px)" }}
      transition={PANEL_TRANSITION}
      className="absolute inset-y-2 right-2 left-2 z-10 flex flex-col overflow-hidden rounded-xl border bg-card shadow-2xl md:left-auto md:w-full md:max-w-panel"
    >
      <header className="flex items-center gap-3 px-6 pt-6">
        <ServiceIcon kind={service.kind} />
        <h2 className="me-auto truncate font-semibold text-xl">{service.name}</h2>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Fermer le panneau"
          render={
            <Link to="/projects/$projectId/$env" params={{ projectId, env: environmentName }} />
          }
        >
          <XIcon />
        </Button>
      </header>
      <PanelTabs projectId={projectId} environmentName={environmentName} serviceId={serviceId} />
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </motion.aside>
  );
}
