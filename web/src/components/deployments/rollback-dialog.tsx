import { useMutation } from "@tanstack/react-query";

import { rollback } from "~/api/deployments";
import type { Deployment, Service } from "~/api/types";
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
          <AlertDialogTitle>Revenir à ce déploiement ?</AlertDialogTitle>
          <AlertDialogDescription>
            {service.name} va redéployer « {deploymentTitle(deployment)} », sans rebuild. Les
            variables simples et de build reviennent avec cette révision ; les secrets gardent leur
            valeur actuelle.
          </AlertDialogDescription>
          {service.volume && (
            <p className="rounded-lg border border-warning/32 bg-warning/8 px-3 py-2 text-sm text-warning-foreground">
              Les données de {service.volume.name} ne reviennent pas en arrière. Après une
              migration, l'ancien code tournera sur le nouveau schéma.
            </p>
          )}
          {mutation.error && (
            <p role="alert" className="text-destructive-foreground text-sm">
              {mutation.error.message}
            </p>
          )}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogClose render={<Button variant="ghost" />}>Annuler</AlertDialogClose>
          <Button loading={mutation.isPending} onClick={() => mutation.mutate()}>
            Revenir à ce déploiement
          </Button>
        </AlertDialogFooter>
      </AlertDialogPopup>
    </AlertDialog>
  );
}
