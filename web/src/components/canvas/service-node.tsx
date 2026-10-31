import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useParams } from "@tanstack/react-router";
import { Handle, type NodeProps, Position } from "@xyflow/react";
import { HardDriveIcon } from "lucide-react";

import { serviceChanges } from "~/api/changes";
import { serviceById } from "~/api/services";
import { ServiceIcon } from "~/components/services/service-icon";
import { ServiceStatusLabel } from "~/components/services/service-status";

// The node reads its service by id, so a status change re-renders the card without rebuilding the
// graph.
export function ServiceNode({ id }: NodeProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(id) });
  const { data: changes } = useLiveSuspenseQuery({ query: serviceChanges(id) });
  // The open panel tells which service is selected.
  const { serviceId } = useParams({ strict: false });
  if (!service) {
    return null;
  }

  return (
    <div className="w-72">
      <div
        data-selected={serviceId === id}
        data-staged={changes.length > 0}
        className="relative flex flex-col gap-6 rounded-xl border bg-card p-4 shadow-sm transition-colors hover:border-ring/40 data-[staged=true]:border-info data-[selected=true]:border-ring data-[staged=true]:ring-2 data-[selected=true]:ring-2 data-[staged=true]:ring-info/30 data-[selected=true]:ring-ring/24"
      >
        {" "}
        <Handle type="target" position={Position.Top} isConnectable={false} className="opacity-0" />
        <div className="flex items-start gap-3">
          <ServiceIcon kind={service.kind} />
          <div className="flex min-w-0 flex-col">
            <span className="truncate font-medium">{service.name}</span>
            <span className="min-h-5 truncate text-muted-foreground text-sm">
              {service.domains[0]}
            </span>
          </div>
        </div>
        <span className="flex items-center justify-between gap-2">
          <ServiceStatusLabel status={service.status} />
          {changes.length > 0 && (
            <span className="text-info-foreground text-xs">{changes.length} en attente</span>
          )}
        </span>
        <Handle
          type="source"
          position={Position.Bottom}
          isConnectable={false}
          className="opacity-0"
        />
      </div>
      {service.volume && (
        <div className="mx-3 flex items-center gap-2 rounded-b-lg border border-t-0 bg-muted px-3 py-2 text-muted-foreground text-sm">
          <HardDriveIcon className="size-4" />
          {service.volume.name}
        </div>
      )}
    </div>
  );
}
