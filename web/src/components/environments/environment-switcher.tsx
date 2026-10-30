import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Link } from "@tanstack/react-router";
import { CheckIcon, ChevronsUpDownIcon } from "lucide-react";

import { projectEnvironments } from "~/api/projects";
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

interface EnvironmentSwitcherProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
}

// Reference: infra/screens/03c-project-environment-switcher.png.
export function EnvironmentSwitcher({ projectId, environmentName }: EnvironmentSwitcherProps) {
  const { data: environments } = useLiveSuspenseQuery({ query: projectEnvironments(projectId) });

  return (
    <Menu>
      <MenuTrigger render={<Button variant="ghost" size="sm" />}>
        {environmentName}
        <ChevronsUpDownIcon />
      </MenuTrigger>
      <MenuPopup align="start">
        <MenuGroup>
          <MenuGroupLabel>Environnements</MenuGroupLabel>
          {environments.map((environment) => (
            <MenuLinkItem
              key={environment.id}
              render={
                <Link
                  to="/projects/$projectId/$env"
                  params={{ projectId, env: environment.name }}
                />
              }
            >
              <CheckIcon
                className={environment.name === environmentName ? undefined : "invisible"}
              />
              {environment.name}
            </MenuLinkItem>
          ))}
        </MenuGroup>
      </MenuPopup>
    </Menu>
  );
}
