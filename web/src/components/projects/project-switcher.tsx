import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Link } from "@tanstack/react-router";
import { ChevronsUpDownIcon } from "lucide-react";

import { allProjects, projectById } from "~/api/projects";
import type { ProjectId } from "~/api/types";
import { Button } from "~/components/ui/button";
import {
  Menu,
  MenuGroup,
  MenuGroupLabel,
  MenuLinkItem,
  MenuPopup,
  MenuTrigger,
} from "~/components/ui/menu";

export function ProjectSwitcher({ projectId }: { readonly projectId: ProjectId }) {
  const { data: project } = useLiveSuspenseQuery({ query: projectById(projectId) });
  const { data: projects } = useLiveSuspenseQuery({ query: allProjects });

  return (
    <Menu>
      <MenuTrigger render={<Button variant="ghost" size="sm" />}>
        {project?.name}
        <ChevronsUpDownIcon />
      </MenuTrigger>
      <MenuPopup align="start">
        <MenuGroup>
          <MenuGroupLabel>Projets</MenuGroupLabel>
          {projects.map((other) => (
            <MenuLinkItem
              key={other.id}
              render={<Link to="/projects/$projectId" params={{ projectId: other.id }} />}
            >
              {other.name}
            </MenuLinkItem>
          ))}
        </MenuGroup>
      </MenuPopup>
    </Menu>
  );
}
