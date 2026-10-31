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
        aria-label="Docker image"
        aria-invalid={invalid || undefined}
        placeholder="ghcr.io/acme/app:1.0"
      />
      {invalid && (
        <p className="text-destructive-foreground text-sm">
          Invalid image reference. For example: redis:7 or ghcr.io/acme/app:1.0.
        </p>
      )}
      <Button type="submit" loading={disabled}>
        Deploy image
      </Button>
    </form>
  );
}
