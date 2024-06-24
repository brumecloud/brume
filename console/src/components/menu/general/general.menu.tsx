import Logo from "@/assets/logo.png";
import { BarChart3, Cog, FolderOpenDot, HardDrive, Satellite } from "lucide-react";
import { Link } from "react-router-dom";

export const GenerateMenu = () => {
  return (
    <div className="flex select-none flex-col gap-y-3">
      <span className="flex items-center gap-x-2">
        <img src={Logo} alt="logo" height={25} width={25} className="rounded" />
        <h2 className="text-sm font-semibold">Brume Cloud</h2>
      </span>
      <div className="flex flex-col gap-y-2">
        <Link
          to="/overview"
          className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <FolderOpenDot
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Overview
        </Link>
        <Link
          to="/compute"
          className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <HardDrive
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Compute
        </Link>
        <Link
          to="/network"
          className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <Satellite
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Network
        </Link>
        <Link
          to="/monitoring"
          className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <BarChart3
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Monitoring
        </Link>
        <Link
          to="/settings"
          className="flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer">
          <Cog
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Settings
        </Link>
      </div>
    </div>
  );
};
