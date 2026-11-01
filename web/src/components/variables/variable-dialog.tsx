import { useMutation } from "@tanstack/react-query";
import { type ReactElement, useState } from "react";

import { stageChange } from "~/api/changes";
import type { ServiceId, VariableKind } from "~/api/types";
import { Callout, CalloutIcon } from "~/components/common/callout";
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
import {
  Select,
  SelectItem,
  SelectPopup,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select";
import { parseVariableKind, parseVariableName, variableWarnings } from "~/helpers/variables";

const KINDS = [
  { value: "plain", label: "Plain variable" },
  { value: "build", label: "Build variable" },
  { value: "secret", label: "Secret" },
] as const;

const KIND_DESCRIPTIONS: Record<VariableKind, string> = {
  plain: "Follows the revision: a rollback restores it.",
  build: "Baked into the artifact: changing its value triggers a new build.",
  secret: "Resolved at deploy time, never copied into a revision.",
};

// What an edit starts from. A secret starts empty: its value never comes back to the browser.
interface VariableDraft {
  readonly name: string;
  readonly kind: VariableKind;
  readonly value: string;
}

interface VariableDialogProps {
  readonly serviceId: ServiceId;
  readonly trigger: ReactElement;
  // Absent: a new variable.
  readonly initial?: VariableDraft;
}

// The variable is staged, not applied: the deploy bar of the canvas applies it.
export function VariableDialog({ serviceId, trigger, initial }: VariableDialogProps) {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger render={trigger} />
      <DialogPopup>
        <VariableForm serviceId={serviceId} initial={initial} onStaged={() => setOpen(false)} />
      </DialogPopup>
    </Dialog>
  );
}

interface VariableFormProps {
  readonly serviceId: ServiceId;
  readonly initial: VariableDraft | undefined;
  readonly onStaged: () => void;
}

// Mounted only while the dialog is open: closing it resets the form.
function VariableForm({ serviceId, initial, onStaged }: VariableFormProps) {
  const [name, setName] = useState(initial?.name ?? "");
  const [value, setValue] = useState(initial?.value ?? "");
  const [kind, setKind] = useState<VariableKind>(initial?.kind ?? "plain");
  const [nameInvalid, setNameInvalid] = useState(false);
  // `gcTime: 0`: the mutation holds the value of a secret, it must not linger in the cache.
  const mutation = useMutation({ mutationFn: stageChange, onSuccess: onStaged, gcTime: 0 });

  return (
    <form
      className="contents"
      onSubmit={(event) => {
        event.preventDefault();
        const validName = parseVariableName(name);
        setNameInvalid(!validName);
        if (validName) {
          mutation.mutate({ serviceId, type: "variable", name: validName, kind, value });
        }
      }}
    >
      <DialogHeader>
        <DialogTitle>{initial ? `Edit ${initial.name}` : "New variable"}</DialogTitle>
        <DialogDescription>
          The change waits for the next deployment of this environment.
        </DialogDescription>
      </DialogHeader>
      <DialogPanel>
        <div className="flex flex-col gap-4">
          <Field>
            <FieldLabel>Name</FieldLabel>
            <Input
              autoFocus={!initial}
              disabled={Boolean(initial)}
              value={name}
              aria-invalid={nameInvalid || undefined}
              placeholder="DATABASE_URL"
              onChange={(event) => setName(event.target.value)}
            />
            {nameInvalid && (
              <p className="text-destructive-foreground text-sm">
                Letters, digits and _ only, not starting with a digit.
              </p>
            )}
          </Field>
          <Field>
            <FieldLabel>Type</FieldLabel>
            <Select
              items={KINDS}
              value={kind}
              onValueChange={(next) => setKind(parseVariableKind(next) ?? "plain")}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectPopup>
                {KINDS.map((item) => (
                  <SelectItem key={item.value} value={item.value}>
                    {item.label}
                  </SelectItem>
                ))}
              </SelectPopup>
            </Select>
            <FieldDescription>{KIND_DESCRIPTIONS[kind]}</FieldDescription>
          </Field>
          <Field>
            <FieldLabel>Value</FieldLabel>
            <Input
              autoFocus={Boolean(initial)}
              value={value}
              type={kind === "secret" ? "password" : "text"}
              onChange={(event) => setValue(event.target.value)}
            />
          </Field>
          {variableWarnings(name, value, kind).map((warning) => (
            <Callout key={warning}>
              <CalloutIcon />
              {warning}
            </Callout>
          ))}
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
          {initial ? "Save" : "Add"}
        </Button>
      </DialogFooter>
    </form>
  );
}
