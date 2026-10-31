import { useNavigate } from "@tanstack/react-router";
import { ChevronDownIcon, LayoutGridIcon, ListIcon } from "lucide-react";

import { Button } from "~/components/ui/button";
import { Menu, MenuPopup, MenuRadioGroup, MenuRadioItem, MenuTrigger } from "~/components/ui/menu";
import { ToggleGroup, ToggleGroupItem } from "~/components/ui/toggle-group";
import { type ProjectsSearch, parseProjectsSearch, projectCountLabel } from "~/helpers/dashboard";

const SORT_LABELS = { activity: "Activité récente", name: "Nom" } as const;

interface ProjectToolbarProps {
  readonly count: number;
  readonly search: ProjectsSearch;
}

export function ProjectToolbar({ count, search }: ProjectToolbarProps) {
  const navigate = useNavigate();
  const sort = search.sort ?? "activity";
  const view = search.view ?? "grid";

  return (
    <div className="flex items-center gap-3 text-sm">
      <span className="text-muted-foreground">{projectCountLabel(count)}</span>
      <Menu>
        <MenuTrigger render={<Button variant="ghost" size="sm" />}>
          Trier : {SORT_LABELS[sort]}
          <ChevronDownIcon />
        </MenuTrigger>
        <MenuPopup align="start">
          <MenuRadioGroup
            value={sort}
            onValueChange={(value) => {
              const next = parseProjectsSearch({ sort: value });
              void navigate({ to: "/projects", search: (previous) => ({ ...previous, ...next }) });
            }}
          >
            <MenuRadioItem value="activity">{SORT_LABELS.activity}</MenuRadioItem>
            <MenuRadioItem value="name">{SORT_LABELS.name}</MenuRadioItem>
          </MenuRadioGroup>
        </MenuPopup>
      </Menu>
      <ToggleGroup
        className="ms-auto"
        variant="outline"
        size="sm"
        value={[view]}
        onValueChange={([value]) => {
          // Clicking the pressed item empties the group: the current view stays.
          const next = parseProjectsSearch({ view: value });
          void navigate({ to: "/projects", search: (previous) => ({ ...previous, ...next }) });
        }}
      >
        <ToggleGroupItem value="grid" aria-label="Grille">
          <LayoutGridIcon />
        </ToggleGroupItem>
        <ToggleGroupItem value="list" aria-label="Liste">
          <ListIcon />
        </ToggleGroupItem>
      </ToggleGroup>
    </div>
  );
}
