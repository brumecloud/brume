import { useNavigate } from "@tanstack/react-router";
import { PauseIcon, PlayIcon, SearchIcon } from "lucide-react";

import type { Service } from "~/api/types";
import { Button } from "~/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "~/components/ui/input-group";
import {
  Select,
  SelectItem,
  SelectPopup,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select";
import { type LogsSearch, logCountLabel, parseLogsSearch } from "~/helpers/logs";

const LEVELS = [
  { value: "all", label: "Tous les niveaux" },
  { value: "warn", label: "Avertissements et erreurs" },
  { value: "error", label: "Erreurs" },
] as const;

interface LogToolbarProps {
  readonly search: LogsSearch;
  readonly services: readonly Service[];
  readonly count: number;
  readonly paused: boolean;
  readonly onTogglePause: () => void;
}

// Every filter lives in the URL, validated by parseLogsSearch.
export function LogToolbar({ search, services, count, paused, onTogglePause }: LogToolbarProps) {
  const navigate = useNavigate({ from: "/projects/$projectId/$env/logs" });
  const serviceItems = [
    { value: "all", label: "Tous les services" },
    ...services.map((service) => ({ value: service.id, label: service.name })),
  ];
  // The whole search is validated again: choosing "all" drops the filter from the URL.
  const update = (changes: Record<string, unknown>) =>
    navigate({ search: parseLogsSearch({ ...search, ...changes }), replace: true });

  return (
    <div className="flex flex-wrap items-center gap-2 p-4 pb-3">
      <InputGroup className="min-w-48 flex-1">
        <InputGroupAddon>
          <SearchIcon />
        </InputGroupAddon>
        <InputGroupInput
          type="search"
          aria-label="Rechercher dans les logs"
          placeholder="Rechercher dans les logs…"
          defaultValue={search.q ?? ""}
          onChange={(event) => update({ q: event.target.value })}
        />
      </InputGroup>
      <Select
        items={LEVELS}
        value={search.level ?? "all"}
        onValueChange={(level) => update({ level })}
      >
        <SelectTrigger className="w-auto" aria-label="Niveau">
          <SelectValue />
        </SelectTrigger>
        <SelectPopup>
          {LEVELS.map((level) => (
            <SelectItem key={level.value} value={level.value}>
              {level.label}
            </SelectItem>
          ))}
        </SelectPopup>
      </Select>
      <Select
        items={serviceItems}
        value={search.service ?? "all"}
        onValueChange={(service) => update({ service })}
      >
        <SelectTrigger className="w-auto" aria-label="Service">
          <SelectValue />
        </SelectTrigger>
        <SelectPopup>
          {serviceItems.map((service) => (
            <SelectItem key={service.value} value={service.value}>
              {service.label}
            </SelectItem>
          ))}
        </SelectPopup>
      </Select>
      <span className="text-muted-foreground text-sm">{logCountLabel(count)}</span>
      <Button variant="outline" size="sm" onClick={onTogglePause}>
        {paused ? <PlayIcon /> : <PauseIcon />}
        {paused ? "Reprendre" : "Pause"}
      </Button>
    </div>
  );
}
