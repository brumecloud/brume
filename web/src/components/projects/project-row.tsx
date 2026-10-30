import { Link } from "@tanstack/react-router";

import { ServiceIcon } from "~/components/services/service-icon";
import {
  type ProjectSummary,
  formatDay,
  projectHealth,
  servicesOnlineLabel,
} from "~/helpers/dashboard";

import { HealthDot } from "./health-dot";

export function ProjectRow({ summary }: { readonly summary: ProjectSummary }) {
  const { project, environment, services, onlineCount } = summary;

  return (
    <Link
      to="/projects/$projectId"
      params={{ projectId: project.id }}
      className="grid grid-cols-1 items-center gap-2 px-4 py-3 transition-colors hover:bg-accent sm:grid-cols-12 sm:gap-4"
    >
      <span className="flex min-w-0 flex-col sm:col-span-4">
        <span className="font-medium">{project.name}</span>
        <span className="truncate text-muted-foreground text-sm">{project.description}</span>
      </span>
      <span className="flex gap-2 text-muted-foreground sm:col-span-2">
        {services.map((service) => (
          <ServiceIcon key={service.id} kind={service.kind} />
        ))}
      </span>
      <span className="flex items-center gap-2 text-muted-foreground text-sm sm:col-span-4">
        <HealthDot health={projectHealth(onlineCount, services.length)} />
        {environment?.name}
        <span aria-hidden="true">·</span>
        {servicesOnlineLabel(onlineCount, services.length)}
      </span>
      <span className="text-muted-foreground text-sm sm:col-span-2 sm:text-end">
        {formatDay(project.updatedAt)}
      </span>
    </Link>
  );
}
