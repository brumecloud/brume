import { useNavigate } from "@tanstack/react-router";
import type { ComponentType } from "react";
import { ChevronRightIcon, DatabaseIcon, GlobeIcon, SquareDashedIcon } from "lucide-react";

import type { NewProjectSource } from "~/api/types";
import { DockerIcon, GitHubIcon } from "~/components/icons/brand-icons";
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
}

const SOURCES: readonly Source[] = [
  { value: "github", label: "Dépôt GitHub", icon: GitHubIcon, step: "github" },
  { value: "database", label: "Base de données", icon: DatabaseIcon, step: "database" },
  { value: "static", label: "Site statique", icon: GlobeIcon, step: "static" },
  { value: "image", label: "Image Docker", icon: DockerIcon, step: "image" },
  { value: "empty", label: "Projet vide", icon: SquareDashedIcon },
];

interface SourceStepProps {
  readonly disabled: boolean;
  readonly onCreate: (source: NewProjectSource) => void;
}

// Reference: infra/screens/02-create-project.png.
export function SourceStep({ disabled, onCreate }: SourceStepProps) {
  const navigate = useNavigate();

  return (
    <Command items={SOURCES}>
      <CommandInput placeholder="Que voulez-vous déployer ?" />
      <CommandPanel>
        <CommandEmpty>Aucune source ne correspond.</CommandEmpty>
        <CommandList>
          {(source: Source) => (
            <CommandItem
              key={source.value}
              value={source}
              disabled={disabled}
              onClick={() =>
                source.step
                  ? navigate({ to: "/projects/new", search: { step: source.step } })
                  : onCreate({ kind: "empty" })
              }
            >
              <source.icon className="size-4" />
              <span className="flex-1">{source.label}</span>
              {source.step && <ChevronRightIcon className="size-4 text-muted-foreground" />}
            </CommandItem>
          )}
        </CommandList>
      </CommandPanel>
    </Command>
  );
}
