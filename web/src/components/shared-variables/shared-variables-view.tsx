import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";
import { EyeIcon, EyeOffIcon, PencilIcon, PlusIcon, Trash2Icon } from "lucide-react";
import { useId, useState } from "react";

import type {
  Environment,
  ProjectId,
  ServiceId,
  SharedVariableLevel,
  SharedVariableResult,
  Variable,
} from "~/api/types";
import { allVariables, revealSecret } from "~/api/variables";
import {
  VariableItem,
  VariableItemActions,
  VariableItemLabel,
  VariableItemName,
  VariableItemValue,
  VariableKindBadge,
} from "~/components/variables/variable-item";
import { Button } from "~/components/ui/button";
import { sharedVariables } from "~/helpers/shared-variables";

import { DeleteSharedVariableDialog } from "./delete-shared-variable-dialog";
import { RedeployPrompt } from "./redeploy-prompt";
import { SharedVariableDialog } from "./shared-variable-dialog";

interface SharedVariablesViewProps {
  readonly projectId: ProjectId;
  readonly environment: Environment;
}

interface RowActions {
  readonly onSaved: (result: SharedVariableResult) => void;
  readonly onDelete: (variable: Variable) => void;
}

// The variables services inherit: the project's, then the current environment's, which override
// them.
export function SharedVariablesView({ projectId, environment }: SharedVariablesViewProps) {
  const { data: variables } = useLiveSuspenseQuery({ query: allVariables });
  // The prompts live here, not in a row: saving or deleting refetches the list, and a row it
  // remounts would take them along.
  const [saved, setSaved] = useState<{
    readonly name: string;
    readonly referencedBy: readonly ServiceId[];
  } | null>(null);
  const [removal, setRemoval] = useState<{
    readonly variable: Variable;
    readonly open: boolean;
  } | null>(null);
  const actions: RowActions = {
    onSaved: ({ variable, referencedBy }) => {
      if (referencedBy.length > 0) {
        setSaved({ name: variable.name, referencedBy });
      }
    },
    onDelete: (variable) => setRemoval({ variable, open: true }),
  };

  return (
    <div className="mx-auto flex w-full max-w-4xl flex-col gap-8 p-6">
      <SharedSection
        title="Project"
        description="Shared by every environment. Secrets only reach the services that reference them."
        level="project"
        ownerId={projectId}
        variables={sharedVariables(variables, "project", projectId)}
        actions={actions}
      />
      <SharedSection
        title={`Environment: ${environment.name}`}
        description={`Shared by the services of ${environment.name}. They override the project's.`}
        level="environment"
        ownerId={environment.id}
        variables={sharedVariables(variables, "environment", environment.id)}
        actions={actions}
      />
      <RedeployPrompt
        name={saved?.name ?? ""}
        serviceIds={saved?.referencedBy ?? null}
        onClose={() => setSaved(null)}
      />
      <DeleteSharedVariableDialog
        variable={removal?.variable ?? null}
        open={removal?.open ?? false}
        onOpenChange={(open) => setRemoval((current) => current && { ...current, open })}
      />
    </div>
  );
}

interface SharedSectionProps {
  readonly title: string;
  readonly description: string;
  readonly level: SharedVariableLevel;
  readonly ownerId: string;
  readonly variables: readonly Variable[];
  readonly actions: RowActions;
}

function SharedSection({
  title,
  description,
  level,
  ownerId,
  variables,
  actions,
}: SharedSectionProps) {
  const titleId = useId();

  return (
    <section aria-labelledby={titleId} className="flex flex-col gap-4">
      <div className="flex items-start gap-3">
        <div className="me-auto">
          <h2 id={titleId} className="font-medium">
            {title}
          </h2>
          <p className="text-muted-foreground text-sm">{description}</p>
        </div>
        <SharedVariableDialog
          level={level}
          ownerId={ownerId}
          taken={variables.map((variable) => variable.name)}
          onSaved={actions.onSaved}
          trigger={
            <Button variant="outline" size="sm">
              <PlusIcon />
              New variable
            </Button>
          }
        />
      </div>
      {variables.length > 0 ? (
        <div className="flex flex-col divide-y overflow-hidden rounded-xl border bg-background">
          {variables.map((variable) => (
            <SharedVariableRow
              key={variable.id}
              level={level}
              variable={variable}
              actions={actions}
            />
          ))}
        </div>
      ) : (
        <p className="text-muted-foreground text-sm">No shared variables yet.</p>
      )}
    </section>
  );
}

interface SharedVariableRowProps {
  readonly level: SharedVariableLevel;
  readonly variable: Variable;
  readonly actions: RowActions;
}

function SharedVariableRow({ level, variable, actions }: SharedVariableRowProps) {
  // The revealed value stays in this row's state: `gcTime: 0` drops it once the row is gone.
  const reveal = useMutation({ mutationFn: () => revealSecret(variable.id), gcTime: 0 });
  const secret = variable.kind === "secret";

  return (
    <VariableItem>
      <VariableItemLabel>
        <VariableItemName>{variable.name}</VariableItemName>
        <VariableKindBadge kind={variable.kind} />
      </VariableItemLabel>
      <VariableItemValue>{secret ? (reveal.data ?? "••••••••") : variable.value}</VariableItemValue>
      <VariableItemActions>
        {secret && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={reveal.data ? `Hide ${variable.name}` : `Reveal ${variable.name}`}
            loading={reveal.isPending}
            onClick={() => (reveal.data ? reveal.reset() : reveal.mutate())}
          >
            {reveal.data ? <EyeOffIcon /> : <EyeIcon />}
          </Button>
        )}
        <SharedVariableDialog
          level={level}
          ownerId={variable.ownerId}
          initial={{ name: variable.name, kind: variable.kind, value: variable.value ?? "" }}
          onSaved={actions.onSaved}
          trigger={
            <Button variant="ghost" size="icon-sm" aria-label={`Edit ${variable.name}`}>
              <PencilIcon />
            </Button>
          }
        />
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={`Delete ${variable.name}`}
          onClick={() => actions.onDelete(variable)}
        >
          <Trash2Icon />
        </Button>
      </VariableItemActions>
    </VariableItem>
  );
}
