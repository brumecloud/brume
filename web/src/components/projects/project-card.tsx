import { Link } from "@tanstack/react-router";

import { ServiceIconTile } from "~/components/services/service-icon";
import { type ProjectSummary, projectHealth, servicesOnlineLabel } from "~/helpers/dashboard";

import { HealthDot } from "./health-dot";

// Reference: infra/screens/01-dashboard.png.
export function ProjectCard({ summary }: { readonly summary: ProjectSummary }) {
  const { project, environment, services, onlineCount } = summary;

  return (
    <Link
      to="/projects/$projectId"
      params={{ projectId: project.id }}
      className="flex flex-col gap-3 rounded-xl border bg-card p-2 transition-colors hover:border-ring/40"
    >
      <div className="flex min-h-12 flex-col px-2 pt-2">
        <span className="font-medium">{project.name}</span>
        <span className="truncate text-muted-foreground text-sm">{project.description}</span>
      </div>
      <div className="flex h-44 flex-wrap content-center items-center justify-center gap-3 rounded-lg border bg-background bg-dots p-6">
        {services.map((service) => (
          <ServiceIconTile key={service.id} kind={service.kind} name={service.name} />
        ))}
      </div>
      <div className="flex items-center gap-2 px-2 pb-1 text-muted-foreground text-xs">
        <HealthDot health={projectHealth(onlineCount, services.length)} />
        {environment?.name}
        <span aria-hidden="true">·</span>
        {servicesOnlineLabel(onlineCount, services.length)}
      </div>
    </Link>
  );
}
