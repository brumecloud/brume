import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Handle, type NodeProps, Position } from "@xyflow/react";
import { HardDriveIcon } from "lucide-react";

import { serviceById } from "~/api/services";
import { ServiceIcon } from "~/components/services/service-icon";
import { ServiceStatusLabel } from "~/components/services/service-status";

// Reference: infra/screens/03a-project-canvas-prod.png. The node reads its service by id,
// so a status change re-renders the card without rebuilding the graph.
export function ServiceNode({ id }: NodeProps) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(id) });
  if (!service) {
    return null;
  }

  return (
    <div className="w-72">
      <div className="relative flex flex-col gap-6 rounded-xl border bg-card p-4 shadow-sm">
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
        <ServiceStatusLabel status={service.status} />
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
