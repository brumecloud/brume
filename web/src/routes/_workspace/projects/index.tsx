import { Link, createFileRoute } from "@tanstack/react-router";
import { PlusIcon } from "lucide-react";

import { environmentsCollection, projectsCollection } from "~/api/projects";
import { servicesCollection } from "~/api/services";
import { ProjectDashboard } from "~/components/projects/project-dashboard";
import { ProjectSearch } from "~/components/projects/project-search";
import { Button } from "~/components/ui/button";
import { parseProjectsSearch } from "~/helpers/dashboard";

export const Route = createFileRoute("/_workspace/projects/")({
  validateSearch: parseProjectsSearch,
  loader: () =>
    Promise.all([
      projectsCollection.preload(),
      environmentsCollection.preload(),
      servicesCollection.preload(),
    ]),
  component: ProjectsPage,
});

function ProjectsPage() {
  const search = Route.useSearch();

  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-8 px-6 py-12">
      <header className="flex flex-wrap items-center gap-4">
        <h1 className="me-auto font-semibold text-3xl">Projets</h1>
        <ProjectSearch query={search.q} />
        <Button render={<Link to="/projects/new" />}>
          <PlusIcon />
          Nouveau
        </Button>
      </header>
      <ProjectDashboard search={search} />
    </div>
  );
}
