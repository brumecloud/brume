import type { DatabaseKind, NewProjectSource } from "~/api/types";
import { ServiceIcon } from "~/components/services/service-icon";
import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
  CommandPanel,
} from "~/components/ui/command";

interface Database {
  readonly value: DatabaseKind;
  readonly label: string;
  readonly description: string;
}

// The three database templates of archi/base/templates.md.
const DATABASES: readonly Database[] = [
  { value: "postgres", label: "PostgreSQL", description: "Relational database" },
  { value: "mongodb", label: "MongoDB", description: "Document database" },
  { value: "redis", label: "Redis", description: "Cache and queues" },
];

interface DatabaseStepProps {
  readonly disabled: boolean;
  readonly onCreate: (source: NewProjectSource) => void;
}

export function DatabaseStep({ disabled, onCreate }: DatabaseStepProps) {
  return (
    <Command items={DATABASES}>
      <CommandInput placeholder="Search databases…" />
      <CommandPanel>
        <CommandEmpty>No matching databases.</CommandEmpty>
        <CommandList>
          {(database: Database) => (
            <CommandItem
              key={database.value}
              value={database}
              disabled={disabled}
              onClick={() => onCreate({ kind: database.value })}
            >
              <ServiceIcon kind={database.value} />
              <span className="flex flex-col">
                <span>{database.label}</span>
                <span className="text-muted-foreground text-xs">{database.description}</span>
              </span>
            </CommandItem>
          )}
        </CommandList>
      </CommandPanel>
    </Command>
  );
}
