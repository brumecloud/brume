import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";
import { EyeIcon, EyeOffIcon, PencilIcon, PlusIcon, Trash2Icon } from "lucide-react";
import { useId, useState } from "react";

import type { Environment, ProjectId, SharedVariableLevel, Variable } from "~/api/types";
import { allVariables, removeSharedVariable, revealSecret } from "~/api/variables";
import {
  AlertDialog,
  AlertDialogClose,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogPopup,
  AlertDialogTitle,
} from "~/components/ui/alert-dialog";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { sharedVariables } from "~/helpers/shared-variables";

import { SharedVariableDialog } from "./shared-variable-dialog";

const KIND_LABELS = { build: "Build", secret: "Secret" } as const;

interface SharedVariablesViewProps {
  readonly projectId: ProjectId;
  readonly environment: Environment;
}

// The variables services inherit: the project's, then the current environment's, which override
// them.
export function SharedVariablesView({ projectId, environment }: SharedVariablesViewProps) {
  const { data: variables } = useLiveSuspenseQuery({ query: allVariables });

  return (
    <div className="mx-auto flex w-full max-w-4xl flex-col gap-8 p-6">
      <SharedSection
        title="Project"
        description="Shared by every environment. Secrets only reach the services that reference them."
        level="project"
        ownerId={projectId}
        variables={sharedVariables(variables, "project", projectId)}
      />
      <SharedSection
        title={`Environment: ${environment.name}`}
        description={`Shared by the services of ${environment.name}. They override the project's.`}
        level="environment"
        ownerId={environment.id}
        variables={sharedVariables(variables, "environment", environment.id)}
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
}

function SharedSection({ title, description, level, ownerId, variables }: SharedSectionProps) {
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
            <SharedVariableRow key={variable.id} variable={variable} />
          ))}
        </div>
      ) : (
        <p className="text-muted-foreground text-sm">No shared variables yet.</p>
      )}
    </section>
  );
}

function SharedVariableRow({ variable }: { readonly variable: Variable }) {
  const [deleting, setDeleting] = useState(false);
  // The revealed value stays in this row's state: `gcTime: 0` drops it once the row is gone.
  const reveal = useMutation({ mutationFn: () => revealSecret(variable.id), gcTime: 0 });
  const secret = variable.kind === "secret";

  return (
    <div className="grid grid-cols-1 items-center gap-2 px-4 py-2.5 sm:grid-cols-12 sm:gap-4">
      <span className="flex min-w-0 items-center gap-2 sm:col-span-5">
        <span className="truncate font-mono text-sm">{variable.name}</span>
        {variable.kind !== "plain" && <Badge variant="outline">{KIND_LABELS[variable.kind]}</Badge>}
      </span>
      <span className="truncate font-mono text-muted-foreground text-sm sm:col-span-5">
        {secret ? (reveal.data ?? "••••••••") : variable.value}
      </span>
      <span className="flex justify-end gap-1 sm:col-span-2">
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
          level={variable.level === "project" ? "project" : "environment"}
          ownerId={variable.ownerId}
          initial={{ name: variable.name, kind: variable.kind, value: variable.value ?? "" }}
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
          onClick={() => setDeleting(true)}
        >
          <Trash2Icon />
        </Button>
      </span>
      <DeleteSharedVariableDialog variable={variable} open={deleting} onOpenChange={setDeleting} />
    </div>
  );
}

interface DeleteSharedVariableDialogProps {
  readonly variable: Variable;
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
}

// The server refuses (409) to delete a secret an active deployment still references.
function DeleteSharedVariableDialog({
  variable,
  open,
  onOpenChange,
}: DeleteSharedVariableDialogProps) {
  const mutation = useMutation({
    mutationFn: () => removeSharedVariable(variable.id),
    onSuccess: () => onOpenChange(false),
  });

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogPopup>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete {variable.name}?</AlertDialogTitle>
          <AlertDialogDescription>
            {variable.kind === "secret"
              ? "Services that still reference it must stop doing so first."
              : "The services that inherit it redeploy without it."}
          </AlertDialogDescription>
          {mutation.error && (
            <p role="alert" className="text-destructive-foreground text-sm">
              {mutation.error.message}
            </p>
          )}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogClose render={<Button variant="ghost" />}>Cancel</AlertDialogClose>
          <Button
            variant="destructive"
            loading={mutation.isPending}
            onClick={() => mutation.mutate()}
          >
            Delete
          </Button>
        </AlertDialogFooter>
      </AlertDialogPopup>
    </AlertDialog>
  );
}
