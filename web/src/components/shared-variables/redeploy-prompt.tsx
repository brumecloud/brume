import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";

import { redeployServices } from "~/api/deployments";
import { allEnvironments } from "~/api/projects";
import { allServices } from "~/api/services";
import type { ServiceId } from "~/api/types";
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
import { groupImpact, referencesImpact } from "~/helpers/shared-variables";

import { ImpactList } from "./impact-dialog";

interface RedeployPromptProps {
  readonly name: string;
  // The services that reference the secret; null when the prompt is closed.
  readonly serviceIds: readonly ServiceId[] | null;
  readonly onClose: () => void;
}

// A shared secret is saved at once, but containers read it at deploy time: the panel offers to
// redeploy what references it.
export function RedeployPrompt({ name, serviceIds, onClose }: RedeployPromptProps) {
  return (
    <AlertDialog open={serviceIds !== null} onOpenChange={(open) => !open && onClose()}>
      <AlertDialogPopup>
        {serviceIds && <RedeployChoice name={name} serviceIds={serviceIds} onClose={onClose} />}
      </AlertDialogPopup>
    </AlertDialog>
  );
}

function RedeployChoice({
  name,
  serviceIds,
  onClose,
}: {
  readonly name: string;
  readonly serviceIds: readonly ServiceId[];
  readonly onClose: () => void;
}) {
  const { data: environments } = useLiveSuspenseQuery({ query: allEnvironments });
  const { data: services } = useLiveSuspenseQuery({ query: allServices });
  const groups = groupImpact(referencesImpact(serviceIds, services), environments, services);
  const mutation = useMutation({
    mutationFn: () => redeployServices(serviceIds),
    onSuccess: onClose,
  });

  return (
    <>
      <AlertDialogHeader>
        <AlertDialogTitle>{name} is saved</AlertDialogTitle>
        <AlertDialogDescription>
          The services that reference it keep the old value until their next deployment.
        </AlertDialogDescription>
        <ImpactList groups={groups} />
        {mutation.error && (
          <p role="alert" className="text-destructive-foreground text-sm">
            {mutation.error.message}
          </p>
        )}
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogClose render={<Button variant="ghost" />}>Later</AlertDialogClose>
        <Button loading={mutation.isPending} onClick={() => mutation.mutate()}>
          Redeploy now
        </Button>
      </AlertDialogFooter>
    </>
  );
}
