import { TriangleAlertIcon } from "lucide-react";
import type { ComponentProps } from "react";

import { cn } from "~/lib/utils";

// A warning to read before acting: a value that looks like a secret, data a rollback leaves
// behind.
export function Callout({ className, ...props }: ComponentProps<"p">) {
  return (
    <p
      className={cn(
        "flex gap-2 rounded-lg border border-warning/32 bg-warning/8 px-3 py-2 text-sm text-warning-foreground",
        className,
      )}
      {...props}
    />
  );
}

export function CalloutIcon({ className, ...props }: ComponentProps<typeof TriangleAlertIcon>) {
  return <TriangleAlertIcon className={cn("mt-0.5 size-4 shrink-0", className)} {...props} />;
}
