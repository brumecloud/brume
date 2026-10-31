import { useNavigate } from "@tanstack/react-router";
import type { ComponentType } from "react";
import { ChevronRightIcon, DatabaseIcon, GlobeIcon, SquareDashedIcon } from "lucide-react";

import type { NewProjectSource } from "~/api/types";
import { DockerIcon, GitHubIcon } from "~/components/icons/brand-icons";
import { Badge } from "~/components/ui/badge";
import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
  CommandPanel,
} from "~/components/ui/command";
import type { NewProjectStep } from "~/helpers/new-project";

interface Source {
  readonly value: string;
  readonly label: string;
  readonly icon: ComponentType<{ className?: string }>;
  readonly step?: NewProjectStep;
  // Shown, not offered: the server cannot build it yet.
  readonly soon?: boolean;
}

const SOURCES: readonly Source[] = [
  { value: "github", label: "GitHub repository", icon: GitHubIcon, step: "github", soon: true },
  { value: "database", label: "Database", icon: DatabaseIcon, step: "database", soon: true },
  { value: "static", label: "Static site", icon: GlobeIcon, step: "static", soon: true },
  { value: "image", label: "Docker image", icon: DockerIcon, step: "image" },
  { value: "empty", label: "Empty project", icon: SquareDashedIcon },
];

interface SourceStepProps {
  readonly disabled: boolean;
  readonly onCreate: (source: NewProjectSource) => void;
}

export function SourceStep({ disabled, onCreate }: SourceStepProps) {
  const navigate = useNavigate();

  return (
    <Command items={SOURCES}>
      <CommandInput placeholder="What do you want to deploy?" />
      <CommandPanel>
        <CommandEmpty>No matching sources.</CommandEmpty>
        <CommandList>
          {(source: Source) => (
            <CommandItem
              key={source.value}
              value={source}
              disabled={disabled || source.soon}
              onClick={() =>
                source.step
                  ? navigate({ to: "/projects/new", search: { step: source.step } })
                  : onCreate({ kind: "empty" })
              }
            >
              <source.icon className="size-4" />
              <span className="flex-1">{source.label}</span>
              {source.soon ? (
                <Badge variant="secondary">Coming soon</Badge>
              ) : (
                source.step && <ChevronRightIcon className="size-4 text-muted-foreground" />
              )}
            </CommandItem>
          )}
        </CommandList>
      </CommandPanel>
    </Command>
  );
}
