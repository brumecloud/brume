import { useMutation } from "@tanstack/react-query";
import { EyeIcon, EyeOffIcon, PencilIcon, Trash2Icon, Undo2Icon } from "lucide-react";

import { stageChange, unstageChange } from "~/api/changes";
import type { ServiceId } from "~/api/types";
import { revealSecret } from "~/api/variables";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import type { VariableDraftRow } from "~/helpers/changes";

import { VariableDialog } from "./variable-dialog";

const KIND_LABELS = { build: "Build", secret: "Secret" } as const;
const ORIGIN_LABELS = { project: "Projet", environment: "Environnement" } as const;
const DRAFT_LABELS = { added: "Nouvelle", modified: "Modifiée", removed: "Supprimée" } as const;

interface VariableRowProps {
  readonly serviceId: ServiceId;
  readonly row: VariableDraftRow;
}

export function VariableRow({ serviceId, row }: VariableRowProps) {
  const { variable, draft, changeId } = row;
  // The revealed value stays in this row's state: `gcTime: 0` drops it once the row is gone.
  const reveal = useMutation({
    mutationFn: () => (variable ? revealSecret(variable.id) : Promise.resolve("")),
    gcTime: 0,
  });
  const removal = useMutation({ mutationFn: stageChange });
  const revealable = row.kind === "secret" && variable && !draft;

  return (
    <div
      data-draft={draft}
      className="grid grid-cols-1 items-center gap-2 px-4 py-2.5 data-[draft=added]:outline-2 data-[draft=modified]:outline-2 data-[draft=added]:-outline-offset-2 data-[draft=modified]:-outline-offset-2 data-[draft=added]:outline-info data-[draft=modified]:outline-info data-[draft=removed]:opacity-60 sm:grid-cols-12 sm:gap-4"
    >
      <span className="flex min-w-0 items-center gap-2 sm:col-span-5">
        <span className="truncate font-mono text-sm in-data-[draft=removed]:line-through">
          {row.name}
        </span>
        {row.kind !== "plain" && <Badge variant="outline">{KIND_LABELS[row.kind]}</Badge>}
        {row.origin !== "service" && <Badge variant="secondary">{ORIGIN_LABELS[row.origin]}</Badge>}
        {draft && <Badge variant="info">{DRAFT_LABELS[draft]}</Badge>}
      </span>
      <span className="truncate font-mono text-muted-foreground text-sm sm:col-span-5">
        {row.kind === "secret" ? (reveal.data ?? "••••••••") : row.value}
      </span>
      <span className="flex justify-end gap-1 sm:col-span-2">
        {revealable && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={reveal.data ? `Masquer ${row.name}` : `Révéler ${row.name}`}
            loading={reveal.isPending}
            onClick={() => (reveal.data ? reveal.reset() : reveal.mutate())}
          >
            {reveal.data ? <EyeOffIcon /> : <EyeIcon />}
          </Button>
        )}
        {changeId ? (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={`Annuler le changement de ${row.name}`}
            onClick={() => unstageChange(changeId)}
          >
            <Undo2Icon />
          </Button>
        ) : (
          row.origin === "service" && (
            <>
              <VariableDialog
                serviceId={serviceId}
                initial={{
                  name: row.name,
                  kind: row.kind,
                  value: row.kind === "secret" ? "" : (row.value ?? ""),
                }}
                trigger={
                  <Button variant="ghost" size="icon-sm" aria-label={`Modifier ${row.name}`}>
                    <PencilIcon />
                  </Button>
                }
              />
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label={`Supprimer ${row.name}`}
                loading={removal.isPending}
                onClick={() =>
                  removal.mutate({ serviceId, type: "variable-removal", name: row.name })
                }
              >
                <Trash2Icon />
              </Button>
            </>
          )
        )}
      </span>
    </div>
  );
}
