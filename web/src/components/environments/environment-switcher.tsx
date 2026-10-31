import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useNavigate } from "@tanstack/react-router";
import { ChevronsUpDownIcon } from "lucide-react";

import { projectEnvironments } from "~/api/projects";
import type { ProjectId } from "~/api/types";
import { SearchSelect } from "~/components/common/search-select";
import { Button } from "~/components/ui/button";

interface EnvironmentSwitcherProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
}

export function EnvironmentSwitcher({ projectId, environmentName }: EnvironmentSwitcherProps) {
  const { data: environments } = useLiveSuspenseQuery({ query: projectEnvironments(projectId) });
  const navigate = useNavigate();

  return (
    <SearchSelect
      items={environments.map((environment) => environment.name)}
      value={environmentName}
      onValueChange={(env) =>
        navigate({ to: "/projects/$projectId/$env", params: { projectId, env } })
      }
      trigger={<Button variant="ghost" size="sm" />}
      label="Environnement"
      searchPlaceholder="Rechercher un environnement…"
      emptyText="Aucun environnement"
    >
      {environmentName}
      <ChevronsUpDownIcon />
    </SearchSelect>
  );
}
