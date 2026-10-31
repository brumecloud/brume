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
    <SettingField label="Repository" staged={staged}>
      <SearchSelect
        items={repositories?.map((repository) => repository.fullName) ?? []}
        value={value}
        onValueChange={onCommit}
        trigger={<Button variant="outline" className="w-full justify-start" />}
        label="Repository"
        searchPlaceholder="Search repositories…"
        emptyText={isPending ? "Loading…" : "No repositories"}
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
      label={`Branch deployed to ${environmentName}`}
      description="A push to this branch triggers a deployment."
      staged={staged}
    >
      <SearchSelect
        items={branches ?? []}
        value={value}
        onValueChange={onCommit}
        trigger={<Button variant="outline" className="w-full justify-start" />}
        label="Branch"
        searchPlaceholder="Search branches…"
        emptyText={isPending ? "Loading…" : "No branches"}
      >
        <GitBranchIcon />
        <span className="flex-1 truncate text-start">{value}</span>
        <ChevronsUpDownIcon />
      </SearchSelect>
    </SettingField>
  );
}
