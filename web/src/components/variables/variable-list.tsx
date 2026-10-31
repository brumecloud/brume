import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { PlusIcon } from "lucide-react";

import { serviceChanges } from "~/api/changes";
import { serviceById } from "~/api/services";
import type { Environment, ServiceId } from "~/api/types";
import { allVariables } from "~/api/variables";
import { Button } from "~/components/ui/button";
import { stagedVariableRows } from "~/helpers/changes";
import { isDatabase } from "~/helpers/services";
import { resolveVariables, variableCountLabel } from "~/helpers/variables";

import { VariableDialog } from "./variable-dialog";
import { VariableRow } from "./variable-row";

interface VariableListProps {
  readonly serviceId: ServiceId;
  readonly environment: Environment;
}

// Reference: infra/screens/04b-service-variables.png. The variables as they will be after the
// next deploy, drafts marked.
export function VariableList({ serviceId, environment }: VariableListProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const { data: variables } = useLiveSuspenseQuery({ query: allVariables });
  const { data: changes } = useLiveSuspenseQuery({ query: serviceChanges(serviceId) });
  if (!service) {
    return null;
  }
  const rows = stagedVariableRows(resolveVariables(variables, service, environment), changes);

  return (
    <div className="flex flex-col gap-4 p-6">
      <div className="flex items-center gap-3">
        <h3 className="me-auto font-medium">{variableCountLabel(rows.length)}</h3>
        {!isDatabase(service.kind) && (
          <VariableDialog
            serviceId={service.id}
            trigger={
              <Button variant="outline" size="sm">
                <PlusIcon />
                Nouvelle variable
              </Button>
            }
          />
        )}
      </div>
      {isDatabase(service.kind) && (
        <p className="text-muted-foreground text-sm">
          Une base créée depuis un template n'accepte que ses réglages typés, ni variable libre ni
          variable partagée.
        </p>
      )}
      {rows.length > 0 && (
        <div className="flex flex-col divide-y overflow-hidden rounded-xl border bg-background">
          {rows.map((row) => (
            <VariableRow key={row.name} serviceId={service.id} row={row} />
          ))}
        </div>
      )}
    </div>
  );
}
