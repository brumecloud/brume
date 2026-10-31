import { Undo2Icon } from "lucide-react";

import { unstageChange } from "~/api/changes";
import type { Service, StagedChange } from "~/api/types";
import { ServiceIcon } from "~/components/services/service-icon";
import { Button } from "~/components/ui/button";
import {
  Dialog,
  DialogDescription,
  DialogHeader,
  DialogPanel,
  DialogPopup,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/dialog";
import { changesByService, describeChange } from "~/helpers/changes";

interface ChangesDialogProps {
  readonly changes: readonly StagedChange[];
  readonly services: readonly Service[];
}

// The staged changes, service by service, each one undoable.
export function ChangesDialog({ changes, services }: ChangesDialogProps) {
  return (
    <Dialog>
      <DialogTrigger render={<Button variant="outline" size="sm" />}>Détails</DialogTrigger>
      <DialogPopup>
        <DialogHeader>
          <DialogTitle>Changements en attente</DialogTitle>
          <DialogDescription>
            Ils s'appliquent ensemble au prochain déploiement : une nouvelle révision par service.
          </DialogDescription>
        </DialogHeader>
        <DialogPanel>
          <div className="flex flex-col gap-5">
            {changesByService(changes, services).map(({ service, changes: own }) => (
              <section key={service.id} className="flex flex-col gap-2">
                <h3 className="flex items-center gap-2 font-medium text-sm">
                  <ServiceIcon kind={service.kind} />
                  {service.name}
                </h3>
                <ul className="flex flex-col divide-y rounded-lg border bg-background">
                  {own.map((change) => (
                    <li key={change.id} className="flex items-center gap-2 px-3 py-2 text-sm">
                      <span className="min-w-0 flex-1 truncate font-mono">
                        {describeChange(change)}
                      </span>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        aria-label="Annuler ce changement"
                        onClick={() => unstageChange(change.id)}
                      >
                        <Undo2Icon />
                      </Button>
                    </li>
                  ))}
                </ul>
              </section>
            ))}
          </div>
        </DialogPanel>
      </DialogPopup>
    </Dialog>
  );
}
