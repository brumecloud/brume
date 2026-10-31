import { useNavigate } from "@tanstack/react-router";
import { SearchIcon } from "lucide-react";

import { InputGroup, InputGroupAddon, InputGroupInput } from "~/components/ui/input-group";
import { normalizeQuery } from "~/helpers/dashboard";

export function ProjectSearch({ query }: { readonly query: string | undefined }) {
  const navigate = useNavigate();

  return (
    <InputGroup className="sm:w-64">
      <InputGroupAddon>
        <SearchIcon />
      </InputGroupAddon>
      <InputGroupInput
        type="search"
        aria-label="Rechercher un projet"
        placeholder="Rechercher un projet…"
        defaultValue={query ?? ""}
        onChange={(event) =>
          navigate({
            to: "/projects",
            search: (previous) => ({ ...previous, q: normalizeQuery(event.target.value) }),
            replace: true,
          })
        }
      />
    </InputGroup>
  );
}
