import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ReactElement, useState } from "react";

import { createService } from "~/api/services";
import type { EnvironmentId, ProjectId } from "~/api/types";
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
  DialogTrigger,
} from "~/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "~/components/ui/field";
import { Input } from "~/components/ui/input";
import { parseImageReference } from "~/helpers/new-project";
import { serviceNameError } from "~/helpers/services";

interface NewServiceDialogProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly environmentId: EnvironmentId;
  readonly trigger: ReactElement;
}

// A service from a public image, deployed at once: no staged change to apply.
export function NewServiceDialog({ trigger, ...target }: NewServiceDialogProps) {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger render={trigger} />
      <DialogPopup>
        <NewServiceForm {...target} onCreated={() => setOpen(false)} />
      </DialogPopup>
    </Dialog>
  );
}

interface NewServiceFormProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly environmentId: EnvironmentId;
  readonly onCreated: () => void;
}

// Mounted only while the dialog is open: closing it resets the form.
function NewServiceForm({
  projectId,
  environmentName,
  environmentId,
  onCreated,
}: NewServiceFormProps) {
  const navigate = useNavigate();
  const [image, setImage] = useState("");
  const [name, setName] = useState("");
  const [imageInvalid, setImageInvalid] = useState(false);
  const [nameError, setNameError] = useState<string | null>(null);
  const mutation = useMutation({
    mutationFn: (input: { readonly image: string; readonly name?: string }) =>
      createService(environmentId, input),
    onSuccess: (service) => {
      onCreated();
      void navigate({
        to: "/projects/$projectId/$env/services/$serviceId/deployments",
        params: { projectId, env: environmentName, serviceId: service.id },
      });
    },
  });

  return (
    <form
      className="contents"
      onSubmit={(event) => {
        event.preventDefault();
        const reference = parseImageReference(image);
        const typed = name.trim();
        const error = serviceNameError(typed);
        setImageInvalid(!reference);
        setNameError(error);
        if (reference && !error) {
          mutation.mutate({ image: reference, ...(typed ? { name: typed } : {}) });
        }
      }}
    >
      <DialogHeader>
        <DialogTitle>New service</DialogTitle>
        <DialogDescription>
          Brume pulls the image and deploys it in {environmentName}.
        </DialogDescription>
      </DialogHeader>
      <DialogPanel>
        <div className="flex flex-col gap-4">
          <Field>
            <FieldLabel>Docker image</FieldLabel>
            <Input
              autoFocus
              value={image}
              aria-invalid={imageInvalid || undefined}
              placeholder="nginx:1.29"
              onChange={(event) => setImage(event.target.value)}
            />
            {imageInvalid && (
              <p className="text-destructive-foreground text-sm">
                Invalid image reference. For example: redis:8 or ghcr.io/acme/app:1.0.
              </p>
            )}
          </Field>
          <Field>
            <FieldLabel>Name</FieldLabel>
            <Input
              value={name}
              aria-invalid={Boolean(nameError) || undefined}
              placeholder="Named after the image"
              onChange={(event) => setName(event.target.value)}
            />
            {nameError ? (
              <p className="text-destructive-foreground text-sm">{nameError}</p>
            ) : (
              <FieldDescription>
                Optional. Its slug becomes the service's host name.
              </FieldDescription>
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
          Add service
        </Button>
      </DialogFooter>
    </form>
  );
}
