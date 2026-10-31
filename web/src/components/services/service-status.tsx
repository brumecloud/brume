import type { ServiceStatus } from "~/api/types";

const STATUSES = {
  online: { label: "Online", dot: "bg-success", text: "text-success-foreground" },
  deploying: {
    label: "Deploying…",
    dot: "bg-info motion-safe:animate-pulse",
    text: "text-info-foreground",
  },
  crashed: { label: "Crashed", dot: "bg-destructive", text: "text-destructive-foreground" },
  stopped: { label: "Stopped", dot: "bg-muted-foreground", text: "text-muted-foreground" },
} satisfies Record<ServiceStatus, { label: string; dot: string; text: string }>;

export function ServiceStatusLabel({ status }: { readonly status: ServiceStatus }) {
  const { label, dot, text } = STATUSES[status];
  return (
    <span className={`flex items-center gap-2 text-sm ${text}`}>
      <span aria-hidden="true" className={`size-2 rounded-full ${dot}`} />
      {label}
    </span>
  );
}
