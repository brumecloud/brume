import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useNavigate } from "@tanstack/react-router";
import { ChevronsUpDownIcon, PlusIcon } from "lucide-react";
import { useState } from "react";

import { projectEnvironments } from "~/api/projects";
import type { ProjectId } from "~/api/types";
import { SearchSelect } from "~/components/common/search-select";
import { Button } from "~/components/ui/button";

import { NewEnvironmentDialog } from "./new-environment-dialog";

interface EnvironmentSwitcherProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
}

export function EnvironmentSwitcher({ projectId, environmentName }: EnvironmentSwitcherProps) {
  const { data: environments } = useLiveSuspenseQuery({ query: projectEnvironments(projectId) });
  const navigate = useNavigate();
  const [creating, setCreating] = useState(false);
  const names = environments.map((environment) => environment.name);

  return (
    <>
      <SearchSelect
        items={names}
        value={environmentName}
        onValueChange={(env) =>
          navigate({ to: "/projects/$projectId/$env", params: { projectId, env } })
        }
        trigger={<Button variant="ghost" size="sm" />}
        label="Environment"
        searchPlaceholder="Search environments…"
        emptyText="No environments"
        action={{ label: "New environment", icon: <PlusIcon />, onSelect: () => setCreating(true) }}
      >
        {environmentName}
        <ChevronsUpDownIcon />
      </SearchSelect>
      <NewEnvironmentDialog
        projectId={projectId}
        taken={names}
        open={creating}
        onOpenChange={setCreating}
      />
    </>
  );
}
