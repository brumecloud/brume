import { useMutation } from "@tanstack/react-query";
import { type ReactElement, useState } from "react";

import type { SharedVariableLevel, SharedVariableResult, VariableKind } from "~/api/types";
import { setSharedVariable } from "~/api/variables";
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
import { sharedVariableNameError, sharedVariableWarnings } from "~/helpers/shared-variables";
import { parseVariableKind } from "~/helpers/variables";

import { ImpactDialog } from "./impact-dialog";

const KINDS = [
  { value: "plain", label: "Plain variable" },
  { value: "build", label: "Build variable" },
  { value: "secret", label: "Secret" },
] as const;

const KIND_DESCRIPTIONS: Record<VariableKind, string> = {
  plain: "Every service that inherits it redeploys with the new value.",
  build: "Every service that inherits it rebuilds, then redeploys.",
  secret: "Only reaches the services that reference it, at their next deployment.",
};

// An edit cannot rename the variable: no name to refuse.
const NO_NAMES: readonly string[] = [];

// What an edit starts from. A secret starts empty: its value never comes back to the browser.
interface SharedVariableDraft {
  readonly name: string;
  readonly kind: VariableKind;
  readonly value: string;
}

interface SharedVariableDialogProps {
  readonly level: SharedVariableLevel;
  readonly ownerId: string;
  readonly trigger: ReactElement;
  // Absent: a new variable.
  readonly initial?: SharedVariableDraft;
  // The names the level already holds: a new variable cannot take one.
  readonly taken?: readonly string[];
  // The caller offers to redeploy what references a secret: a prompt kept here would go with the
  // row that holds this dialog.
  readonly onSaved: (result: SharedVariableResult) => void;
}

// Unlike a service variable, a shared one is applied at once: a plain or build value after the
// admin saw the services it redeploys, a secret right away.
export function SharedVariableDialog({
  level,
  ownerId,
  trigger,
  initial,
  taken = NO_NAMES,
  onSaved,
}: SharedVariableDialogProps) {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger render={trigger} />
      <DialogPopup>
        <SharedVariableForm
          level={level}
          ownerId={ownerId}
          initial={initial}
          taken={taken}
          onSaved={(result) => {
            setOpen(false);
            onSaved(result);
          }}
        />
      </DialogPopup>
    </Dialog>
  );
}

interface SharedVariableFormProps {
  readonly level: SharedVariableLevel;
  readonly ownerId: string;
  readonly initial: SharedVariableDraft | undefined;
  readonly taken: readonly string[];
  readonly onSaved: (result: SharedVariableResult) => void;
}

// Mounted only while the dialog is open: closing it resets the form.
function SharedVariableForm({ level, ownerId, initial, taken, onSaved }: SharedVariableFormProps) {
  const [name, setName] = useState(initial?.name ?? "");
  const [value, setValue] = useState(initial?.value ?? "");
  const [kind, setKind] = useState<VariableKind>(initial?.kind ?? "plain");
  const [nameError, setNameError] = useState<string | null>(null);
  // A secret's edit starts empty: saving it untouched would wipe the secret.
  const [valueMissing, setValueMissing] = useState(false);
  const [confirming, setConfirming] = useState(false);
  // `gcTime: 0`: the input holds the value of a secret, it must not linger in the cache.
  const mutation = useMutation({ mutationFn: setSharedVariable, onSuccess: onSaved, gcTime: 0 });
  const save = () => mutation.mutate({ level, ownerId, name, kind, value });

  return (
    <>
      <form
        className="contents"
        onSubmit={(event) => {
          event.preventDefault();
          const error = sharedVariableNameError(name, taken);
          const missing = kind === "secret" && value === "";
          setNameError(error);
          setValueMissing(missing);
          if (error || missing) {
            return;
          }
          if (kind === "secret") {
            save();
          } else {
            setConfirming(true);
          }
        }}
      >
        <DialogHeader>
          <DialogTitle>{initial ? `Edit ${initial.name}` : "New shared variable"}</DialogTitle>
          <DialogDescription>
            {level === "project"
              ? "Shared by every environment of the project."
              : "Shared by every service of this environment."}
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
                aria-invalid={nameError !== null || undefined}
                placeholder="LOG_LEVEL"
                onChange={(event) => setName(event.target.value)}
              />
              {nameError && <p className="text-destructive-foreground text-sm">{nameError}</p>}
            </Field>
            <Field>
              <FieldLabel>Type</FieldLabel>
              <Select
                items={KINDS}
                value={kind}
                disabled={Boolean(initial)}
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
                aria-invalid={valueMissing || undefined}
                onChange={(event) => setValue(event.target.value)}
              />
              {valueMissing && (
                <p className="text-destructive-foreground text-sm">A secret needs a value.</p>
              )}
            </Field>
            {sharedVariableWarnings(level, name, value, kind).map((warning) => (
              <Callout key={warning}>
                <CalloutIcon />
                {warning}
              </Callout>
            ))}
            {mutation.error && !confirming && (
              <p role="alert" className="text-destructive-foreground text-sm">
                {mutation.error.message}
              </p>
            )}
          </div>
        </DialogPanel>
        <DialogFooter>
          <DialogClose render={<Button variant="ghost" />}>Cancel</DialogClose>
          <Button type="submit" loading={mutation.isPending && !confirming}>
            {initial ? "Save" : "Add"}
          </Button>
        </DialogFooter>
      </form>
      <ImpactDialog
        open={confirming}
        onOpenChange={setConfirming}
        level={level}
        ownerId={ownerId}
        name={name}
        pending={mutation.isPending}
        error={mutation.error}
        onConfirm={save}
      />
    </>
  );
}
