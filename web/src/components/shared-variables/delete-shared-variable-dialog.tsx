import { useMutation } from "@tanstack/react-query";

import type { Variable } from "~/api/types";
import { removeSharedVariable } from "~/api/variables";
import {
  AlertDialog,
  AlertDialogClose,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogPopup,
  AlertDialogTitle,
} from "~/components/ui/alert-dialog";
import { Button } from "~/components/ui/button";
import { Spinner } from "~/components/ui/spinner";
import { impactSummary } from "~/helpers/shared-variables";
import { useVariableImpact } from "~/hooks/use-variable-impact";

import { ImpactList } from "./impact-dialog";

interface DeleteSharedVariableDialogProps {
  // The last variable asked for: it stays while the dialog closes.
  readonly variable: Variable | null;
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
}

// archi/base/13-modele-de-donnees.md#modifier-une-valeur-partagée: deleting a plain or build value
// redeploys the services that inherit it, so the admin sees them first. The server refuses (409)
// to delete a secret an active deployment still references.
export function DeleteSharedVariableDialog({
  variable,
  open,
  onOpenChange,
}: DeleteSharedVariableDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogPopup>
        {variable && (
          <DeleteConfirmation variable={variable} onDeleted={() => onOpenChange(false)} />
        )}
      </AlertDialogPopup>
    </AlertDialog>
  );
}

// Mounted only while the dialog is open: the impact is asked for when the admin opens it.
function DeleteConfirmation({
  variable,
  onDeleted,
}: {
  readonly variable: Variable;
  readonly onDeleted: () => void;
}) {
  const secret = variable.kind === "secret";
  const { impact, groups, count } = useVariableImpact(
    variable.level === "project" ? "project" : "environment",
    variable.ownerId,
    variable.name,
    !secret,
  );
  const mutation = useMutation({
    mutationFn: () => removeSharedVariable(variable.id),
    onSuccess: onDeleted,
  });
  const failure = impact.error ?? mutation.error;

  return (
    <>
      <AlertDialogHeader>
        <AlertDialogTitle>Delete {variable.name}?</AlertDialogTitle>
        <AlertDialogDescription>
          {secret
            ? "Services that still reference it must stop doing so first."
            : impact.data
              ? impactSummary(count)
              : "Looking for the services it reaches…"}
        </AlertDialogDescription>
        {!secret && impact.isPending && <Spinner />}
        {groups.length > 0 && <ImpactList groups={groups} />}
        {failure && (
          <p role="alert" className="text-destructive-foreground text-sm">
            {failure.message}
          </p>
        )}
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogClose render={<Button variant="ghost" />}>Cancel</AlertDialogClose>
        <Button
          variant="destructive"
          disabled={!secret && (!impact.data || impact.isFetching)}
          loading={mutation.isPending}
          onClick={() => mutation.mutate()}
        >
          Delete
        </Button>
      </AlertDialogFooter>
    </>
  );
}
