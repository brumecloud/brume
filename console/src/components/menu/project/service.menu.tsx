import { CollapsibleWrapper } from "@/components/collapsable";
import type { Service } from "@/schemas/service.schema";
import { cn } from "@/utils";
import { Hammer, ServerCog, Cog, NotepadText } from "lucide-react";
import { Link, NavLink } from "react-router-dom";

type ServiceViewProps = {
  projectId: string;
  service: Service;
};

export const ServiceMenu = ({
  projectId,
  service: { name, id },
}: ServiceViewProps) => {
  const itemClassname = cn("flex flex-col gap-1 py-1 rounded");

  return (
    <CollapsibleWrapper title={name} indent={0}>
      <div className={itemClassname}>
        <NavLink
          to={`/${projectId}/services/${id}/builder`}
          end
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }>
          {({ isActive }) => (
            <>
              <Hammer
                strokeWidth={isActive ? 1.9 : 1.5}
                height={20}
              />
              Builder
            </>
          )}
        </NavLink>
      </div>
      <div className={itemClassname}>
        <NavLink
          to={`/${projectId}/services/${id}/runner`}
          end
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }>
          {({ isActive }) => (
            <>
              <ServerCog
                strokeWidth={isActive ? 2 : 1.5}
                height={20}
              />
              Runner
            </>
          )}
        </NavLink>
      </div>
      {/* <div className={itemClassname}>
        <Link
          to={`/${projectId}/services/${id}/network`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <Network strokeWidth={1.5} height={20} />
          Network
        </Link>
      </div> */}
      {/* <div className={itemClassname}>
        <Link
          to={`/${projectId}/services/${id}/logs`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <NotepadText strokeWidth={1.5} height={20} />
          Logs
        </Link>
      </div> */}
      <div className={itemClassname}>
        <NavLink
          to={`/${projectId}/services/${id}/settings`}
          end
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }>
          {({ isActive }) => (
            <>
              <Cog strokeWidth={isActive ? 1.9 : 1.5} height={20} />
              Settings
            </>
          )}
        </NavLink>
      </div>
    </CollapsibleWrapper>
  );
};
