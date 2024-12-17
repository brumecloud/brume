import { type Machine } from "@/schemas/machine.schema";

export function MachineCard({ machine }: { machine: Machine }) {
  return (
    <div className="rounded-lg border bg-card p-6 text-card-foreground shadow-sm">
      <div className="flex flex-col space-y-1.5">
        <h3 className="text-2xl font-semibold leading-none tracking-tight">
          {machine.name}
        </h3>
        <p className="text-sm text-muted-foreground">{machine.ip}</p>
      </div>
    </div>
  );
}
