import { useQuery } from "@tanstack/react-query";
import { ChevronsUpDownIcon, GitBranchIcon } from "lucide-react";

import { branchesQuery, repositoriesQuery } from "~/api/github";
import { SearchSelect } from "~/components/common/search-select";
import { GitHubIcon } from "~/components/icons/brand-icons";
import { Button } from "~/components/ui/button";

import { SettingField } from "./setting-field";

interface SourceSettingProps {
  readonly value: string;
  readonly staged: boolean;
  readonly onCommit: (value: string) => void;
}

// Picked among the repositories the GitHub App can read.
export function RepositorySetting({ value, staged, onCommit }: SourceSettingProps) {
  const { data: repositories, isPending } = useQuery(repositoriesQuery);

  return (
    <SettingField label="Dépôt" staged={staged}>
      <SearchSelect
        items={repositories?.map((repository) => repository.fullName) ?? []}
        value={value}
        onValueChange={onCommit}
        trigger={<Button variant="outline" className="w-full justify-start" />}
        label="Dépôt"
        searchPlaceholder="Rechercher un dépôt…"
        emptyText={isPending ? "Chargement…" : "Aucun dépôt"}
      >
        <GitHubIcon />
        <span className="flex-1 truncate text-start">{value}</span>
        <ChevronsUpDownIcon />
      </SearchSelect>
    </SettingField>
  );
}

interface BranchSettingProps extends SourceSettingProps {
  readonly repository: string;
  readonly environmentName: string;
}

export function BranchSetting({
  repository,
  environmentName,
  value,
  staged,
  onCommit,
}: BranchSettingProps) {
  const { data: branches, isPending } = useQuery(branchesQuery(repository));

  return (
    <SettingField
      label={`Branche déployée en ${environmentName}`}
      description="Un push sur cette branche déclenche un déploiement."
      staged={staged}
    >
      <SearchSelect
        items={branches ?? []}
        value={value}
        onValueChange={onCommit}
        trigger={<Button variant="outline" className="w-full justify-start" />}
        label="Branche"
        searchPlaceholder="Rechercher une branche…"
        emptyText={isPending ? "Chargement…" : "Aucune branche"}
      >
        <GitBranchIcon />
        <span className="flex-1 truncate text-start">{value}</span>
        <ChevronsUpDownIcon />
      </SearchSelect>
    </SettingField>
  );
}
