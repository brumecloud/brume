import type { HTMLAttributes } from "react";
import { cn } from "@/utils";

type StatusProps = React.HTMLAttributes<HTMLDivElement> & {
  status: "ok" | "cancelled" | "error" | "pending" | "warning";
};

export function Status({ status, ...props }: StatusProps) {
  const color = {
    ok: "bg-green-500",
    cancelled: "bg-red-500",
    error: "bg-red-500",
    pending: "bg-yellow-500",
    warning: "bg-yellow-500",
  };

  return (
    <div className={cn("rounded-full border p-1", color[status])} {...props} />
  );
}

export type StatusIndicatorProps = HTMLAttributes<HTMLSpanElement>;

export const StatusIndicator = ({
  className,
  ...props
}: StatusIndicatorProps) => (
  <span className={cn("relative flex h-2 w-2", className)} {...props}>
    <span
      className={cn(
        "absolute inline-flex h-full w-full animate-ping rounded-full opacity-75",
        "group-[.online]:bg-emerald-500",
        "group-[.offline]:bg-red-500",
        "group-[.maintenance]:bg-blue-500",
        "group-[.degraded]:bg-amber-500"
      )}
    />
    <span
      className={cn(
        "relative inline-flex h-2 w-2 rounded-full",
        "group-[.online]:bg-emerald-500",
        "group-[.offline]:bg-red-500",
        "group-[.maintenance]:bg-blue-500",
        "group-[.degraded]:bg-amber-500"
      )}
    />
  </span>
);
