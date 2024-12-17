import { MachineCard } from "@/components/cards/machine";
import { useMachine } from "@/hooks/useMachine";

export default function Machines() {
  const { machines } = useMachine();

  return (
    <div className="flex flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row items-center justify-between pt-16">
          <div className="flex flex-col pb-8">
            <h2 className="pb-2 font-heading text-3xl">Machines</h2>
            <p>
              Manage your army of machine running your applications
            </p>
          </div>
        </div>
        <div className="flex flex-col gap-4">
          {machines?.map((machine) => (
            <MachineCard key={machine.id} machine={machine} />
          ))}
        </div>
      </div>
    </div>
  );
}
