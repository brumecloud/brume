import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import { createEnvironment } from "~/api/projects";
import type { ProjectId } from "~/api/types";
import { Button } from "~/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogPanel,
  DialogPopup,
  DialogTitle,
} from "~/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "~/components/ui/field";
import { Input } from "~/components/ui/input";
import { environmentNameError } from "~/helpers/environments";

interface NewEnvironmentDialogProps {
  readonly projectId: ProjectId;
  // Names of the project's environments.
  readonly taken: readonly string[];
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
}

export function NewEnvironmentDialog({
  projectId,
  taken,
  open,
  onOpenChange,
}: NewEnvironmentDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogPopup>
        <NewEnvironmentForm
          projectId={projectId}
          taken={taken}
          onCreated={() => onOpenChange(false)}
        />
      </DialogPopup>
    </Dialog>
  );
}

interface NewEnvironmentFormProps {
  readonly projectId: ProjectId;
  readonly taken: readonly string[];
  readonly onCreated: () => void;
}

// Mounted only while the dialog is open: closing it resets the form.
function NewEnvironmentForm({ projectId, taken, onCreated }: NewEnvironmentFormProps) {
  const navigate = useNavigate();
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const mutation = useMutation({
    mutationFn: (env: string) => createEnvironment(projectId, env),
    onSuccess: (environment) => {
      onCreated();
      void navigate({
        to: "/projects/$projectId/$env",
        params: { projectId, env: environment.name },
      });
    },
  });

  return (
    <form
      className="contents"
      onSubmit={(event) => {
        event.preventDefault();
        const nameError = environmentNameError(name, taken);
        setError(nameError);
        if (!nameError) {
          mutation.mutate(name);
        }
      }}
    >
      <DialogHeader>
        <DialogTitle>New environment</DialogTitle>
        <DialogDescription>
          It starts empty, on its own network. Add services to it from the canvas.
        </DialogDescription>
      </DialogHeader>
      <DialogPanel>
        <div className="flex flex-col gap-4">
          <Field>
            <FieldLabel>Name</FieldLabel>
            <Input
              autoFocus
              value={name}
              aria-invalid={Boolean(error) || undefined}
              placeholder="staging"
              onChange={(event) => setName(event.target.value)}
            />
            {error ? (
              <p className="text-destructive-foreground text-sm">{error}</p>
            ) : (
              <FieldDescription>Also the environment's URL segment.</FieldDescription>
            )}
          </Field>
          {mutation.error && (
            <p role="alert" className="text-destructive-foreground text-sm">
              {mutation.error.message}
            </p>
          )}
        </div>
      </DialogPanel>
      <DialogFooter>
        <DialogClose render={<Button variant="ghost" />}>Cancel</DialogClose>
        <Button type="submit" loading={mutation.isPending}>
          Create
        </Button>
      </DialogFooter>
    </form>
  );
}
