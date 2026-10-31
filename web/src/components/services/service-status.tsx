import type { ServiceStatus } from "~/api/types";

const STATUSES = {
  online: { label: "En ligne", dot: "bg-success", text: "text-success-foreground" },
  deploying: {
    label: "Déploiement…",
    dot: "bg-info motion-safe:animate-pulse",
    text: "text-info-foreground",
  },
  crashed: { label: "En panne", dot: "bg-destructive", text: "text-destructive-foreground" },
  stopped: { label: "Arrêté", dot: "bg-muted-foreground", text: "text-muted-foreground" },
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
