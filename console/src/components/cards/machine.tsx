import { type Machine } from "@/schemas/machine.schema";
import { Activity } from "lucide-react";

export function MachineCard({ machine }: { machine: Machine }) {
  return (
    <div className="flex h-full w-full flex-col justify-end rounded-lg border bg-card p-6 text-card-foreground shadow-sm">
      <div className="flex flex-col space-y-1.5">
        <div className="flex flex-row items-center justify-between">
          <h3 className="font-regular text-xl leading-none tracking-tight">
            {machine.name}
          </h3>
          <div className="flex flex-row items-center justify-center">
            <Activity className="h-4 w-4 stroke-green-300 text-muted-foreground" />
          </div>
        </div>
        <p className="text-sm text-muted-foreground">{machine.ip}</p>
      </div>
    </div>
  );
}
