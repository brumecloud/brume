import type { ComponentProps } from "react";

import type { VariableKind } from "~/api/types";
import { Badge } from "~/components/ui/badge";
import { cn } from "~/lib/utils";

const KIND_LABELS = { build: "Build", secret: "Secret" } as const;

// One line of a variable list: the name and its badges, the value, then the actions.
export function VariableItem({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn(
        "grid grid-cols-1 items-center gap-2 px-4 py-2.5 sm:grid-cols-12 sm:gap-4",
        className,
      )}
      {...props}
    />
  );
}

// The name and the badges that follow it.
export function VariableItemLabel({ className, ...props }: ComponentProps<"span">) {
  return (
    <span className={cn("flex min-w-0 items-center gap-2 sm:col-span-5", className)} {...props} />
  );
}

export function VariableItemName({ className, ...props }: ComponentProps<"span">) {
  return <span className={cn("truncate font-mono text-sm", className)} {...props} />;
}

export function VariableItemValue({ className, ...props }: ComponentProps<"span">) {
  return (
    <span
      className={cn("truncate font-mono text-muted-foreground text-sm sm:col-span-5", className)}
      {...props}
    />
  );
}

export function VariableItemActions({ className, ...props }: ComponentProps<"span">) {
  return <span className={cn("flex justify-end gap-1 sm:col-span-2", className)} {...props} />;
}

// A plain variable goes without a badge.
export function VariableKindBadge({ kind }: { readonly kind: VariableKind }) {
  return kind === "plain" ? null : <Badge variant="outline">{KIND_LABELS[kind]}</Badge>;
}
