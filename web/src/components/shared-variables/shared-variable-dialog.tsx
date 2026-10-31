import { useMutation } from "@tanstack/react-query";
import { TriangleAlertIcon } from "lucide-react";
import { type ReactElement, useState } from "react";

import type {
  ServiceId,
  SharedVariableLevel,
  SharedVariableResult,
  VariableKind,
} from "~/api/types";
import { setSharedVariable } from "~/api/variables";
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
import { sharedVariableWarnings } from "~/helpers/shared-variables";
import { parseVariableKind, parseVariableName } from "~/helpers/variables";

import { ImpactDialog } from "./impact-dialog";
import { RedeployPrompt } from "./redeploy-prompt";

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
}

// Unlike a service variable, a shared one is applied at once: a plain or build value after the
// admin saw the services it redeploys, a secret right away.
export function SharedVariableDialog({
  level,
  ownerId,
  trigger,
  initial,
}: SharedVariableDialogProps) {
  const [open, setOpen] = useState(false);
  const [saved, setSaved] = useState<{ name: string; referencedBy: readonly ServiceId[] } | null>(
    null,
  );

  return (
    <>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogTrigger render={trigger} />
        <DialogPopup>
          <SharedVariableForm
            level={level}
            ownerId={ownerId}
            initial={initial}
            onSaved={({ variable, referencedBy }) => {
              setOpen(false);
              if (referencedBy.length > 0) {
                setSaved({ name: variable.name, referencedBy });
              }
            }}
          />
        </DialogPopup>
      </Dialog>
      <RedeployPrompt
        name={saved?.name ?? ""}
        serviceIds={saved?.referencedBy ?? null}
        onClose={() => setSaved(null)}
      />
    </>
  );
}

interface SharedVariableFormProps {
  readonly level: SharedVariableLevel;
  readonly ownerId: string;
  readonly initial: SharedVariableDraft | undefined;
  readonly onSaved: (result: SharedVariableResult) => void;
}

// Mounted only while the dialog is open: closing it resets the form.
function SharedVariableForm({ level, ownerId, initial, onSaved }: SharedVariableFormProps) {
  const [name, setName] = useState(initial?.name ?? "");
  const [value, setValue] = useState(initial?.value ?? "");
  const [kind, setKind] = useState<VariableKind>(initial?.kind ?? "plain");
  const [nameInvalid, setNameInvalid] = useState(false);
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
          const validName = parseVariableName(name);
          const missing = kind === "secret" && value === "";
          setNameInvalid(!validName);
          setValueMissing(missing);
          if (!validName || missing) {
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
                aria-invalid={nameInvalid || undefined}
                placeholder="LOG_LEVEL"
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
              <p
                key={warning}
                className="flex gap-2 rounded-lg border border-warning/32 bg-warning/8 px-3 py-2 text-sm text-warning-foreground"
              >
                <TriangleAlertIcon className="mt-0.5 size-4 shrink-0" />
                {warning}
              </p>
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
