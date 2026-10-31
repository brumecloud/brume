import { useMutation } from "@tanstack/react-query";

import { rollback } from "~/api/deployments";
import type { Deployment, Service } from "~/api/types";
import { Callout } from "~/components/common/callout";
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
import { deploymentTitle } from "~/helpers/deployments";

interface RollbackDialogProps {
  readonly service: Service;
  readonly deployment: Deployment;
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
}

// archi/base/13-modele-de-donnees.md#rollback: plain variables come back with the revision,
// secrets keep their current value, data never goes back.
export function RollbackDialog({ service, deployment, open, onOpenChange }: RollbackDialogProps) {
  const mutation = useMutation({
    mutationFn: () => rollback(deployment.id),
    onSuccess: () => onOpenChange(false),
  });

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogPopup>
        <AlertDialogHeader>
          <AlertDialogTitle>Roll back to this deployment?</AlertDialogTitle>
          <AlertDialogDescription>
            {service.name} will redeploy “{deploymentTitle(deployment)}” without a rebuild. Plain
            and build variables come back with this revision; secrets keep their current value.
          </AlertDialogDescription>
          {service.volume && (
            <Callout>
              The data in {service.volume.name} does not roll back. After a migration, the old code
              will run against the new schema.
            </Callout>
          )}
          {mutation.error && (
            <p role="alert" className="text-destructive-foreground text-sm">
              {mutation.error.message}
            </p>
          )}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogClose render={<Button variant="ghost" />}>Cancel</AlertDialogClose>
          <Button loading={mutation.isPending} onClick={() => mutation.mutate()}>
            Roll back
          </Button>
        </AlertDialogFooter>
      </AlertDialogPopup>
    </AlertDialog>
  );
}
