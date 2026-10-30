import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import { deleteService } from "~/api/services";
import type { ProjectId, Service } from "~/api/types";
import {
  AlertDialog,
  AlertDialogClose,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogPopup,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "~/components/ui/alert-dialog";
import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";

interface DeleteServiceDialogProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly service: Service;
}

// Destructive and irreversible: the admin types the name of the service to confirm.
export function DeleteServiceDialog({
  projectId,
  environmentName,
  service,
}: DeleteServiceDialogProps) {
  const navigate = useNavigate();
  const [typed, setTyped] = useState("");
  const mutation = useMutation({
    mutationFn: () => deleteService(service.id),
    onSuccess: () =>
      navigate({ to: "/projects/$projectId/$env", params: { projectId, env: environmentName } }),
  });

  return (
    <AlertDialog>
      <AlertDialogTrigger render={<Button variant="destructive-outline" size="sm" />}>
        Supprimer le service
      </AlertDialogTrigger>
      <AlertDialogPopup>
        <AlertDialogHeader>
          <AlertDialogTitle>Supprimer {service.name} ?</AlertDialogTitle>
          <AlertDialogDescription>
            Le container s'arrête et ses déploiements disparaissent.
            {service.volume &&
              ` Le volume ${service.volume.name} est supprimé avec ses données.`}{" "}
            Tapez {service.name} pour confirmer.
          </AlertDialogDescription>
          <Input
            aria-label="Nom du service"
            value={typed}
            onChange={(event) => setTyped(event.target.value)}
          />
          {mutation.error && (
            <p role="alert" className="text-destructive-foreground text-sm">
              {mutation.error.message}
            </p>
          )}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogClose render={<Button variant="ghost" />}>Annuler</AlertDialogClose>
          <Button
            variant="destructive"
            disabled={typed !== service.name}
            loading={mutation.isPending}
            onClick={() => mutation.mutate()}
          >
            Supprimer
          </Button>
        </AlertDialogFooter>
      </AlertDialogPopup>
    </AlertDialog>
  );
}
