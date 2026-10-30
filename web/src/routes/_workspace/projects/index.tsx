import { createFileRoute } from "@tanstack/react-router";

import { projectsCollection } from "~/api/projects";
import { ProjectList } from "~/components/projects/project-list";

export const Route = createFileRoute("/_workspace/projects/")({
  loader: () => projectsCollection.preload(),
  component: ProjectsPage,
});

function ProjectsPage() {
  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-12">
      <h1 className="font-semibold text-3xl">Projets</h1>
      <ProjectList />
    </div>
  );
}
