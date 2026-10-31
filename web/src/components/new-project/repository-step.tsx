import { useSuspenseQuery } from "@tanstack/react-query";

import { repositoriesQuery } from "~/api/github";
import type { NewProjectSource, Repository } from "~/api/types";
import { GitHubIcon } from "~/components/icons/brand-icons";
import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
  CommandPanel,
} from "~/components/ui/command";

interface RepositoryStepProps {
  readonly kind: "github" | "static";
  readonly disabled: boolean;
  readonly onCreate: (source: NewProjectSource) => void;
}

export function RepositoryStep({ kind, disabled, onCreate }: RepositoryStepProps) {
  const { data: repositories } = useSuspenseQuery(repositoriesQuery);

  return (
    <Command
      items={repositories}
      // Base UI hands back the items given above.
      itemToStringValue={(item) => (item as Repository).fullName}
    >
      <CommandInput placeholder="Rechercher un dépôt…" />
      <CommandPanel>
        <CommandEmpty>Aucun dépôt ne correspond.</CommandEmpty>
        <CommandList>
          {(repository: Repository) => (
            <CommandItem
              key={repository.fullName}
              value={repository}
              disabled={disabled}
              onClick={() => onCreate({ kind, repository: repository.fullName })}
            >
              <GitHubIcon className="size-4" />
              {repository.fullName}
            </CommandItem>
          )}
        </CommandList>
      </CommandPanel>
    </Command>
  );
}
