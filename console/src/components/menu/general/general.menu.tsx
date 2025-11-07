import { useQuery } from "@apollo/client";
import { Cog, FolderOpenDot, Store } from "lucide-react";
import { NavLink } from "react-router-dom";
import Logo from "@/assets/logo.png";
import { ME_QUERY } from "@/gql/user.graphql";
import { cn } from "@/utils";

export const GenerateMenu = () => {
  const { data: me, loading } = useQuery(ME_QUERY, {
    fetchPolicy: "cache-only",
  });

  return (
    <div className="flex select-none flex-col gap-y-3">
      <span className="flex items-center gap-x-2">
        <img alt="logo" className="rounded" height={25} src={Logo} width={25} />
        <h2 className="font-semibold text-sm">{me?.me?.organization?.name}</h2>
      </span>
      <div className="flex flex-col gap-y-2">
        <NavLink
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }
          to="/overview"
        >
          {({ isActive }) => (
            <>
              <FolderOpenDot
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm",
                  isActive &&
                    "bg-gradient-to-r from-[#aac8e6] to-[#437ae1] text-white"
                )}
                height={20}
                strokeWidth={1.5}
              />
              Stacks
            </>
          )}
        </NavLink>
        <NavLink
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }
          to="/marketplace"
        >
          {({ isActive }) => (
            <>
              <Store
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm",
                  isActive &&
                    "bg-gradient-to-r from-[#aac8e6] to-[#437ae1] text-white"
                )}
                height={20}
                strokeWidth={1.5}
              />
              Marketplace
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
        {/* <NavLink
          to="/monitoring"
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }
        >
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
        </NavLink> */}
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
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-2 text-sm hover:cursor-pointer",
              isActive && "font-medium"
            )
          }
          to="/settings"
        >
          {({ isActive }) => (
            <>
              <Cog
                className={cn(
                  "h-6 w-6 rounded-sm border bg-white/80 p-[3px] shadow-sm transition-all",
                  isActive &&
                    "bg-gradient-to-r from-[#dfa6d8] to-[#c3226d] text-white"
                )}
                height={20}
                strokeWidth={1.5}
              />
              Settings
            </>
          )}
        </NavLink>
      </div>
    </div>
  );
};
