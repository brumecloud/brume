import { useState } from "react";

import type { NewProjectSource } from "~/api/types";
import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { parseImageReference } from "~/helpers/new-project";

interface ImageStepProps {
  readonly disabled: boolean;
  readonly onCreate: (source: NewProjectSource) => void;
}

export function ImageStep({ disabled, onCreate }: ImageStepProps) {
  const [invalid, setInvalid] = useState(false);

  return (
    <form
      className="flex flex-col gap-3 p-4"
      onSubmit={(event) => {
        event.preventDefault();
        const image = parseImageReference(String(new FormData(event.currentTarget).get("image")));
        setInvalid(!image);
        if (image) {
          onCreate({ kind: "image", image });
        }
      }}
    >
      <Input
        name="image"
        autoFocus
        aria-label="Image Docker"
        aria-invalid={invalid || undefined}
        placeholder="ghcr.io/organisation/app:1.0"
      />
      {invalid && (
        <p className="text-destructive-foreground text-sm">
          Référence d'image invalide. Exemple : redis:7 ou ghcr.io/organisation/app:1.0.
        </p>
      )}
      <Button type="submit" loading={disabled}>
        Déployer l'image
      </Button>
    </form>
  );
}
