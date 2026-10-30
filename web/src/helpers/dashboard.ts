import type { Environment, Project, Service } from "~/api/types";

import { defaultEnvironment } from "./projects";

const MAX_QUERY_LENGTH = 100;
const DAY_FORMAT = new Intl.DateTimeFormat("fr-FR", {
  day: "numeric",
  month: "short",
  year: "numeric",
});

type ProjectsView = "grid" | "list";
type ProjectsSort = "activity" | "name";

export type ProjectHealth = "empty" | "healthy" | "degraded";

// Absent values mean the defaults: grid view, most recent activity first.
export interface ProjectsSearch {
  readonly view?: ProjectsView | undefined;
  readonly sort?: ProjectsSort | undefined;
  readonly q?: string | undefined;
}

export interface ProjectSummary {
  readonly project: Project;
  readonly environment: Environment | undefined;
  readonly services: readonly Service[];
  readonly onlineCount: number;
}

export function parseProjectsSearch(raw: Record<string, unknown>): ProjectsSearch {
  return {
    ...(raw.view === "grid" || raw.view === "list" ? { view: raw.view } : {}),
    ...(raw.sort === "activity" || raw.sort === "name" ? { sort: raw.sort } : {}),
    ...(typeof raw.q === "string" && normalizeQuery(raw.q) ? { q: normalizeQuery(raw.q) } : {}),
  };
}

export function normalizeQuery(raw: string): string | undefined {
  return raw.trim() ? raw.slice(0, MAX_QUERY_LENGTH) : undefined;
}

export function projectSummaries(
  projects: readonly Project[],
  environments: readonly Environment[],
  services: readonly Service[],
  search: ProjectsSearch,
): ProjectSummary[] {
  const query = search.q?.trim().toLowerCase() ?? "";
  return projects
    .filter((project) => `${project.name} ${project.description}`.toLowerCase().includes(query))
    .toSorted(search.sort === "name" ? byName : byRecentActivity)
    .map((project) => {
      const environment = defaultEnvironment(environments, project.id);
      const own = services
        .filter((service) => service.environmentId === environment?.id)
        .toSorted((a, b) => a.name.localeCompare(b.name));
      return {
        project,
        environment,
        services: own,
        onlineCount: own.filter((service) => service.status === "online").length,
      };
    });
}

function byName(a: Project, b: Project): number {
  return a.name.localeCompare(b.name);
}

function byRecentActivity(a: Project, b: Project): number {
  return Date.parse(b.updatedAt) - Date.parse(a.updatedAt);
}

export function projectCountLabel(count: number): string {
  if (count === 0) {
    return "Aucun projet";
  }
  return `${count} ${count > 1 ? "projets" : "projet"}`;
}

export function projectHealth(online: number, total: number): ProjectHealth {
  if (total === 0) {
    return "empty";
  }
  return online === total ? "healthy" : "degraded";
}

export function servicesOnlineLabel(online: number, total: number): string {
  if (total === 0) {
    return "Aucun service";
  }
  return `${online}/${total} ${total > 1 ? "services" : "service"} en ligne`;
}

export function formatDay(iso: string): string {
  return DAY_FORMAT.format(new Date(iso));
}
