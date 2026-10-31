import type { SharedVariableLevel } from "~/api/types";
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
import { type ImpactGroup, impactConfirmLabel, impactSummary } from "~/helpers/shared-variables";
import { useVariableImpact } from "~/hooks/use-variable-impact";

interface ImpactDialogProps {
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
  readonly level: SharedVariableLevel;
  readonly ownerId: string;
  readonly name: string;
  readonly pending: boolean;
  readonly error: Error | null;
  readonly onConfirm: () => void;
}

// archi/base/13-modele-de-donnees.md#modifier-une-valeur-partagée: before a shared plain or build
// value changes, the admin sees every service it redeploys, production included.
export function ImpactDialog({ open, onOpenChange, ...impact }: ImpactDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogPopup>
        <ImpactConfirmation {...impact} />
      </AlertDialogPopup>
    </AlertDialog>
  );
}

// Mounted only while the dialog is open: the impact is asked for when the admin confirms.
function ImpactConfirmation({
  level,
  ownerId,
  name,
  pending,
  error,
  onConfirm,
}: Omit<ImpactDialogProps, "open" | "onOpenChange">) {
  const { impact, groups, count } = useVariableImpact(level, ownerId, name, true);
  const failure = impact.error ?? error;

  return (
    <>
      <AlertDialogHeader>
        <AlertDialogTitle>Change {name}?</AlertDialogTitle>
        <AlertDialogDescription>
          {impact.data ? impactSummary(count) : "Looking for the services it reaches…"}
        </AlertDialogDescription>
        {impact.isPending && <Spinner />}
        {groups.length > 0 && <ImpactList groups={groups} />}
        {failure && (
          <p role="alert" className="text-destructive-foreground text-sm">
            {failure.message}
          </p>
        )}
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogClose render={<Button variant="ghost" />}>Cancel</AlertDialogClose>
        <Button disabled={!impact.data || impact.isFetching} loading={pending} onClick={onConfirm}>
          {impactConfirmLabel(count)}
        </Button>
      </AlertDialogFooter>
    </>
  );
}

// The services a change touches, one line per environment.
export function ImpactList({ groups }: { readonly groups: readonly ImpactGroup[] }) {
  return (
    <dl className="flex flex-col divide-y rounded-lg border text-sm">
      {groups.map((group) => (
        <div key={group.environmentName} className="flex gap-3 px-3 py-2">
          <dt className="w-24 shrink-0 text-muted-foreground">{group.environmentName}</dt>
          <dd className="min-w-0">{group.services.join(", ")}</dd>
        </div>
      ))}
    </dl>
  );
}
