import { useMutation } from "@tanstack/react-query";
import { EyeIcon, EyeOffIcon, PencilIcon, Trash2Icon, Undo2Icon } from "lucide-react";

import { stageChange, unstageChange } from "~/api/changes";
import type { ServiceId } from "~/api/types";
import { revealSecret } from "~/api/variables";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import type { VariableDraftRow } from "~/helpers/changes";

import { VariableDialog } from "./variable-dialog";
import {
  VariableItem,
  VariableItemActions,
  VariableItemLabel,
  VariableItemName,
  VariableItemValue,
  VariableKindBadge,
} from "./variable-item";

const ORIGIN_LABELS = { project: "Project", environment: "Environment" } as const;
const DRAFT_LABELS = { added: "New", modified: "Modified", removed: "Deleted" } as const;

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
  // A reference has no value of its own: the shared secret resolves at deploy time.
  const shared = Boolean(variable?.reference) && !draft;
  const revealable = row.kind === "secret" && variable && !draft && !shared;

  return (
    <VariableItem
      data-draft={draft}
      className="data-[draft=added]:outline-2 data-[draft=modified]:outline-2 data-[draft=added]:-outline-offset-2 data-[draft=modified]:-outline-offset-2 data-[draft=added]:outline-info data-[draft=modified]:outline-info data-[draft=removed]:opacity-60"
    >
      <VariableItemLabel>
        <VariableItemName className="in-data-[draft=removed]:line-through">
          {row.name}
        </VariableItemName>
        <VariableKindBadge kind={row.kind} />
        {row.origin !== "service" && <Badge variant="secondary">{ORIGIN_LABELS[row.origin]}</Badge>}
        {shared && <Badge variant="secondary">Shared</Badge>}
        {draft && <Badge variant="info">{DRAFT_LABELS[draft]}</Badge>}
      </VariableItemLabel>
      <VariableItemValue>
        {shared
          ? "Resolved at deploy time"
          : row.kind === "secret"
            ? (reveal.data ?? "••••••••")
            : row.value}
      </VariableItemValue>
      <VariableItemActions>
        {revealable && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={reveal.data ? `Hide ${row.name}` : `Reveal ${row.name}`}
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
            aria-label={`Discard the change to ${row.name}`}
            onClick={() => unstageChange(changeId)}
          >
            <Undo2Icon />
          </Button>
        ) : (
          row.origin === "service" && (
            <>
              {!shared && (
                <VariableDialog
                  serviceId={serviceId}
                  initial={{
                    name: row.name,
                    kind: row.kind,
                    value: row.kind === "secret" ? "" : (row.value ?? ""),
                  }}
                  trigger={
                    <Button variant="ghost" size="icon-sm" aria-label={`Edit ${row.name}`}>
                      <PencilIcon />
                    </Button>
                  }
                />
              )}
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label={`Delete ${row.name}`}
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
      </VariableItemActions>
    </VariableItem>
  );
}
