import { useMutation } from "@tanstack/react-query";
import { TriangleAlertIcon } from "lucide-react";
import { type ReactElement, useState } from "react";

import { stageChange } from "~/api/changes";
import type { ServiceId, VariableKind } from "~/api/types";
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
  { value: "plain", label: "Variable simple" },
  { value: "build", label: "Variable de build" },
  { value: "secret", label: "Secret" },
] as const;

const KIND_DESCRIPTIONS: Record<VariableKind, string> = {
  plain: "Suit la révision : un rollback la restaure.",
  build: "Finit dans l'artefact : changer sa valeur relance un build.",
  secret: "Résolu au déploiement, jamais copié dans une révision.",
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
  const mutation = useMutation({ mutationFn: stageChange, onSuccess: onStaged });

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
        <DialogTitle>{initial ? `Modifier ${initial.name}` : "Nouvelle variable"}</DialogTitle>
        <DialogDescription>
          Le changement attend le prochain déploiement de cet environnement.
        </DialogDescription>
      </DialogHeader>
      <DialogPanel>
        <div className="flex flex-col gap-4">
          <Field>
            <FieldLabel>Nom</FieldLabel>
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
                Lettres, chiffres et _ seulement, sans commencer par un chiffre.
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
            <FieldLabel>Valeur</FieldLabel>
            <Input
              autoFocus={Boolean(initial)}
              value={value}
              type={kind === "secret" ? "password" : "text"}
              onChange={(event) => setValue(event.target.value)}
            />
          </Field>
          {variableWarnings(name, value, kind).map((warning) => (
            <p
              key={warning}
              className="flex gap-2 rounded-lg border border-warning/32 bg-warning/8 px-3 py-2 text-sm text-warning-foreground"
            >
              <TriangleAlertIcon className="mt-0.5 size-4 shrink-0" />
              {warning}
            </p>
          ))}
          {mutation.error && (
            <p role="alert" className="text-destructive-foreground text-sm">
              {mutation.error.message}
            </p>
          )}
        </div>
      </DialogPanel>
      <DialogFooter>
        <DialogClose render={<Button variant="ghost" />}>Annuler</DialogClose>
        <Button type="submit" loading={mutation.isPending}>
          {initial ? "Modifier" : "Ajouter"}
        </Button>
      </DialogFooter>
    </form>
  );
}
