import { Satellite, ShieldHalf } from "lucide-react";

export const NetworkMenu = () => (
  <div className="flex select-none flex-col gap-y-3">
    <h2 className="text-gray-500 text-sm">Network</h2>
    <div className="flex flex-col gap-y-2">
      <div className="flex select-none flex-row items-center gap-1 text-sm hover:cursor-pointer">
        <Satellite strokeWidth={1.5} height={20} />
        Mesh
      </div>
      <div className="flex select-none flex-row items-center gap-1 text-sm hover:cursor-pointer">
        <ShieldHalf strokeWidth={1.5} height={20} />
        VPN
      </div>
    </div>
  </div>
);
