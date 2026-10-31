import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { FolderSearchIcon } from "lucide-react";

import { allEnvironments, allProjects } from "~/api/projects";
import { allServices } from "~/api/services";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";
import { type ProjectsSearch, projectSummaries } from "~/helpers/dashboard";

import { ProjectCard } from "./project-card";
import { ProjectRow } from "./project-row";
import { ProjectToolbar } from "./project-toolbar";

export function ProjectDashboard({ search }: { readonly search: ProjectsSearch }) {
  const { data: projects } = useLiveSuspenseQuery({ query: allProjects });
  const { data: environments } = useLiveSuspenseQuery({ query: allEnvironments });
  const { data: services } = useLiveSuspenseQuery({ query: allServices });
  const summaries = projectSummaries(projects, environments, services, search);

  return (
    <div className="flex flex-col gap-6">
      <ProjectToolbar count={summaries.length} search={search} />
      {summaries.length === 0 && (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <FolderSearchIcon />
            </EmptyMedia>
            <EmptyTitle>Aucun projet</EmptyTitle>
            <EmptyDescription>Aucun projet ne correspond à cette recherche.</EmptyDescription>
          </EmptyHeader>
        </Empty>
      )}
      {search.view === "list" ? (
        <div className="flex flex-col divide-y overflow-hidden rounded-xl border bg-card">
          {summaries.map((summary) => (
            <ProjectRow key={summary.project.id} summary={summary} />
          ))}
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {summaries.map((summary) => (
            <ProjectCard key={summary.project.id} summary={summary} />
          ))}
        </div>
      )}
    </div>
  );
}
