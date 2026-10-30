import type { DeploymentStatus } from "~/api/types";
import { Badge } from "~/components/ui/badge";

const BADGES = {
  active: { label: "Actif", variant: "success" },
  building: { label: "Build", variant: "info" },
  deploying: { label: "Déploiement", variant: "info" },
  failed: { label: "Échec", variant: "error" },
  superseded: { label: "Remplacé", variant: "outline" },
} as const satisfies Record<DeploymentStatus, { label: string; variant: string }>;

export function DeploymentStatusBadge({ status }: { readonly status: DeploymentStatus }) {
  const { label, variant } = BADGES[status];
  return <Badge variant={variant}>{label}</Badge>;
}
