import Logo from "@/assets/logo.png";
import { cn } from "@/utils";
import { BarChart3, Cog, FolderOpenDot, HardDrive, Satellite } from "lucide-react";
import { Link, NavLink } from "react-router-dom";

export const GenerateMenu = () => {
  return (
    <div className="flex select-none flex-col gap-y-3">
      <span className="flex items-center gap-x-2">
        <img src={Logo} alt="logo" height={25} width={25} className="rounded" />
        <h2 className="text-sm font-semibold">Brume Cloud</h2>
      </span>
      <div className="flex flex-col gap-y-2">
        <NavLink
          to="/overview"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          <FolderOpenDot
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Overview
        </NavLink>
        <NavLink
          to="/compute"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          <HardDrive
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Compute
        </NavLink>
        <NavLink
          to="/network"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          <Satellite
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Network
        </NavLink>
        <NavLink
          to="/monitoring"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          <BarChart3
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Monitoring
        </NavLink>
        <NavLink
          to="/settings"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          <Cog
            strokeWidth={1.5}
            height={20}
            className="h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm"
          />
          Settings
        </NavLink>
      </div>
    </div>
  );
};
