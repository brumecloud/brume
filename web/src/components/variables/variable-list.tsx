import { useLiveSuspenseQuery } from "@tanstack/react-db";

import { serviceById } from "~/api/services";
import type { Environment, ServiceId } from "~/api/types";
import { allVariables } from "~/api/variables";
import { isDatabase } from "~/helpers/services";
import { resolveVariables, variableCountLabel } from "~/helpers/variables";

import { NewVariableDialog } from "./new-variable-dialog";
import { VariableRow } from "./variable-row";

interface VariableListProps {
  readonly serviceId: ServiceId;
  readonly environment: Environment;
}

// Reference: infra/screens/04b-service-variables.png.
export function VariableList({ serviceId, environment }: VariableListProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const { data: variables } = useLiveSuspenseQuery({ query: allVariables });
  if (!service) {
    return null;
  }
  const resolved = resolveVariables(variables, service, environment);

  return (
    <div className="flex flex-col gap-4 p-6">
      <div className="flex items-center gap-3">
        <h3 className="me-auto font-medium">{variableCountLabel(resolved.length)}</h3>
        {!isDatabase(service.kind) && <NewVariableDialog serviceId={service.id} />}
      </div>
      {isDatabase(service.kind) && (
        <p className="text-muted-foreground text-sm">
          Une base créée depuis un template n'accepte que ses réglages typés, ni variable libre ni
          variable partagée.
        </p>
      )}
      {resolved.length > 0 && (
        <div className="flex flex-col divide-y rounded-xl border bg-background">
          {resolved.map((entry) => (
            <VariableRow key={entry.variable.id} resolved={entry} />
          ))}
        </div>
      )}
    </div>
  );
}
