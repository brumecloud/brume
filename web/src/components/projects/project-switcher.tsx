import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useNavigate } from "@tanstack/react-router";
import { ChevronsUpDownIcon } from "lucide-react";

import { allProjects, projectById } from "~/api/projects";
import type { Project, ProjectId } from "~/api/types";
import { SearchSelect } from "~/components/common/search-select";
import { Button } from "~/components/ui/button";

const projectName = (project: Project) => project.name;
const sameProject = (project: Project, value: Project) => project.id === value.id;

export function ProjectSwitcher({ projectId }: { readonly projectId: ProjectId }) {
  const { data: project } = useLiveSuspenseQuery({ query: projectById(projectId) });
  const { data: projects } = useLiveSuspenseQuery({ query: allProjects });
  const navigate = useNavigate();

  return (
    <SearchSelect
      items={projects}
      value={project ?? null}
      onValueChange={(other) =>
        navigate({ to: "/projects/$projectId", params: { projectId: other.id } })
      }
      itemLabel={projectName}
      isSameItem={sameProject}
      trigger={<Button variant="ghost" size="sm" />}
      label="Projet"
      searchPlaceholder="Rechercher un projet…"
      emptyText="Aucun projet"
    >
      {project?.name}
      <ChevronsUpDownIcon />
    </SearchSelect>
  );
}
