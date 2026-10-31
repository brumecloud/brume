import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";
import { Link2Icon, PlusIcon } from "lucide-react";

import { serviceChanges, stageChange } from "~/api/changes";
import { serviceById } from "~/api/services";
import type { Environment, ServiceId } from "~/api/types";
import { allVariables } from "~/api/variables";
import { SearchSelect } from "~/components/common/search-select";
import { Button } from "~/components/ui/button";
import { stagedVariableRows } from "~/helpers/changes";
import { referenceableSecrets } from "~/helpers/shared-variables";
import { isDatabase } from "~/helpers/services";
import { resolveVariables, variableCountLabel } from "~/helpers/variables";

import { VariableDialog } from "./variable-dialog";
import { VariableRow } from "./variable-row";

interface VariableListProps {
  readonly serviceId: ServiceId;
  readonly environment: Environment;
}

// The variables as they will be after the next deploy, drafts marked.
export function VariableList({ serviceId, environment }: VariableListProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const { data: variables } = useLiveSuspenseQuery({ query: allVariables });
  const { data: changes } = useLiveSuspenseQuery({ query: serviceChanges(serviceId) });
  // A reference is staged like any variable: a secret without a value, resolved at deploy time.
  const reference = useMutation({ mutationFn: stageChange });
  if (!service) {
    return null;
  }
  const rows = stagedVariableRows(resolveVariables(variables, service, environment), changes);
  const secrets = referenceableSecrets(
    variables,
    environment,
    rows.map((row) => row.name),
  );

  return (
    <div className="flex flex-col gap-4 p-6">
      <div className="flex items-center gap-3">
        <h3 className="me-auto font-medium">{variableCountLabel(rows.length)}</h3>
        {!isDatabase(service.kind) && (
          <SearchSelect
            items={secrets}
            value={null}
            onValueChange={(name) =>
              reference.mutate({
                serviceId: service.id,
                type: "variable",
                name,
                kind: "secret",
                value: null,
              })
            }
            trigger={<Button variant="outline" size="sm" loading={reference.isPending} />}
            label="Reference a shared secret"
            searchPlaceholder="Search shared secrets…"
            emptyText="No shared secrets to reference"
          >
            <Link2Icon />
            Shared secret
          </SearchSelect>
        )}
        {!isDatabase(service.kind) && (
          <VariableDialog
            serviceId={service.id}
            trigger={
              <Button variant="outline" size="sm">
                <PlusIcon />
                New variable
              </Button>
            }
          />
        )}
      </div>
      {isDatabase(service.kind) && (
        <p className="text-muted-foreground text-sm">
          A database created from a template only takes its typed settings: no free-form or shared
          variables.
        </p>
      )}
      {reference.error && (
        <p role="alert" className="text-destructive-foreground text-sm">
          {reference.error.message}
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
