import { HardDrive } from "lucide-react";

export const ComputeMenu = () => {
  return (
    <div className="flex select-none flex-col gap-y-3 pb-2">
      <h2 className="text-sm text-gray-500">Compute</h2>
      <div className="flex flex-col gap-y-2">
        <div className="flex select-none flex-row items-center gap-1 text-sm hover:cursor-pointer">
          <HardDrive strokeWidth={1.5} height={20} />
          Machines
        </div>
      </div>
    </div>
  );
};
