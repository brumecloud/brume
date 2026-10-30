import { useMutation } from "@tanstack/react-query";
import { EyeIcon, EyeOffIcon, Trash2Icon } from "lucide-react";

import { deleteVariable, revealSecret } from "~/api/variables";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import type { ResolvedVariable } from "~/helpers/variables";

const KIND_LABELS = { build: "Build", secret: "Secret" } as const;
const ORIGIN_LABELS = { project: "Projet", environment: "Environnement" } as const;

export function VariableRow({ resolved }: { readonly resolved: ResolvedVariable }) {
  const { variable, origin } = resolved;
  // The revealed value stays in this row's state: `gcTime: 0` drops it once the row is gone.
  const reveal = useMutation({ mutationFn: () => revealSecret(variable.id), gcTime: 0 });

  return (
    <div className="grid grid-cols-1 items-center gap-2 px-4 py-2.5 sm:grid-cols-12 sm:gap-4">
      <span className="flex min-w-0 items-center gap-2 sm:col-span-5">
        <span className="truncate font-mono text-sm">{variable.name}</span>
        {variable.kind !== "plain" && <Badge variant="outline">{KIND_LABELS[variable.kind]}</Badge>}
        {origin !== "service" && <Badge variant="secondary">{ORIGIN_LABELS[origin]}</Badge>}
      </span>
      <span className="truncate font-mono text-muted-foreground text-sm sm:col-span-5">
        {variable.kind === "secret" ? (reveal.data ?? "••••••••") : variable.value}
      </span>
      <span className="flex justify-end gap-1 sm:col-span-2">
        {variable.kind === "secret" && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={reveal.data ? `Masquer ${variable.name}` : `Révéler ${variable.name}`}
            loading={reveal.isPending}
            onClick={() => (reveal.data ? reveal.reset() : reveal.mutate())}
          >
            {reveal.data ? <EyeOffIcon /> : <EyeIcon />}
          </Button>
        )}
        {origin === "service" && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={`Supprimer ${variable.name}`}
            onClick={() => deleteVariable(variable.id)}
          >
            <Trash2Icon />
          </Button>
        )}
      </span>
    </div>
  );
}
