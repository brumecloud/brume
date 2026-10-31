import type { DeploymentStatus } from "~/api/types";
import { Badge } from "~/components/ui/badge";

const BADGES = {
  active: { label: "Active", variant: "success" },
  building: { label: "Building", variant: "info" },
  deploying: { label: "Deploying", variant: "info" },
  failed: { label: "Failed", variant: "error" },
  superseded: { label: "Superseded", variant: "outline" },
} as const satisfies Record<DeploymentStatus, { label: string; variant: string }>;

export function DeploymentStatusBadge({ status }: { readonly status: DeploymentStatus }) {
  const { label, variant } = BADGES[status];
  return <Badge variant={variant}>{label}</Badge>;
}
