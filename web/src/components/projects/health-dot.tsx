import type { ProjectHealth } from "~/helpers/dashboard";

const COLORS: Record<ProjectHealth, string> = {
  empty: "bg-muted-foreground",
  healthy: "bg-success",
  degraded: "bg-warning",
};

export function HealthDot({ health }: { readonly health: ProjectHealth }) {
  return <span aria-hidden="true" className={`size-1.5 shrink-0 rounded-full ${COLORS[health]}`} />;
}
