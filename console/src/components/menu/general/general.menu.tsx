import Logo from "@/assets/logo.png";
import { cn } from "@/utils";
import {
  BarChart3,
  Cog,
  FolderOpenDot,
  HardDrive,
} from "lucide-react";
import { NavLink } from "react-router-dom";

export const GenerateMenu = () => {
  return (
    <div className="flex select-none flex-col gap-y-3">
      <span className="flex items-center gap-x-2">
        <img
          src={Logo}
          alt="logo"
          height={25}
          width={25}
          className="rounded"
        />
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
          {({ isActive }) => (
            <>
              <FolderOpenDot
                strokeWidth={1.5}
                height={20}
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm",
                  isActive &&
                    "bg-gradient-to-r from-[#aac8e6] to-[#437ae1] text-white"
                )}
              />
              Overview
            </>
          )}
        </NavLink>
        <NavLink
          to="/machines"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          {({ isActive }) => (
            <>
              <HardDrive
                strokeWidth={1.5}
                height={20}
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm",
                  isActive &&
                    "bg-gradient-to-r from-[#f0bc81] to-[#ee8866] text-white"
                )}
              />
              Machines
            </>
          )}
        </NavLink>
        {/* <NavLink
          to="/network"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          {({ isActive }) => (
            <>
              <Satellite
                strokeWidth={1.5}
                height={20}
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm transition-all",
                  isActive &&
                    "bg-gradient-to-r from-[#9be4ad] to-[#61b531] text-white"
                )}
              />
              Network
            </>
          )}
        </NavLink> */}
        <NavLink
          to="/monitoring"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          {({ isActive }) => (
            <>
              <BarChart3
                strokeWidth={1.5}
                height={20}
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm transition-all",
                  isActive &&
                    "bg-gradient-to-r from-[#b9aae6] to-[#8a66ee] text-white"
                )}
              />
              Monitoring
            </>
          )}
        </NavLink>
        {/* <NavLink
          to="/ai"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          {({ isActive }) => (
            <>
              <Brain
                strokeWidth={1.5}
                height={20}
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm transition-all",
                  isActive &&
                    "bg-gradient-to-r from-[#dbaae6] to-[#c366ee] text-white"
                )}
              />
              AI
            </>
          )}
        </NavLink> */}
        <NavLink
          to="/settings"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }>
          {({ isActive }) => (
            <>
              <Cog
                strokeWidth={1.5}
                height={20}
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm transition-all",
                  isActive &&
                    "bg-gradient-to-r from-[#dfa6d8] to-[#c3226d] text-white"
                )}
              />
              Settings
            </>
          )}
        </NavLink>
      </div>
    </div>
  );
};
