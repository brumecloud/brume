import Logo from "@/assets/logo.png";
import { BarChart3, Cog, SquareActivity } from "lucide-react";

export const GenerateMenu = () => {
  return (
    <div className="flex select-none flex-col gap-y-3">
      <span className="flex items-center gap-x-2">
        <img src={Logo} alt="logo" height={25} width={25} className="rounded" />
        <h2 className="text-sm font-semibold">Brume Cloud</h2>
      </span>
      <div className="flex flex-col gap-y-2">
        <div className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <BarChart3
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Monitoring
        </div>
        <div className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <SquareActivity
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Activity
        </div>
        <div className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <Cog
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Settings
        </div>
      </div>
    </div>
  );
};
