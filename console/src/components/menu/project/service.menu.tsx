import { Cog, FolderGit2, Hammer, ServerCog } from "lucide-react";
import { NavLink } from "react-router-dom";
import { CollapsibleWrapper } from "@/components/collapsable";
import type { Service } from "@/schemas/service.schema";
import { cn } from "@/utils";

type ServiceViewProps = {
  projectId: string;
  service: Service;
};

export const ServiceMenu = ({
  projectId,
  service: { name, id },
}: ServiceViewProps) => {
  const itemClassname = cn("flex flex-col gap-1 rounded py-1");

  return (
    <CollapsibleWrapper indent={0} title={name}>
      <div className={itemClassname}>
        <NavLink
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }
          end
          to={`/${projectId}/services/${id}/source`}
        >
          {({ isActive }) => (
            <>
              <FolderGit2 height={20} strokeWidth={isActive ? 1.9 : 1.5} />
              Source
            </>
          )}
        </NavLink>
      </div>
      <div className={itemClassname}>
        <NavLink
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }
          end
          to={`/${projectId}/services/${id}/builder`}
        >
          {({ isActive }) => (
            <>
              <Hammer height={20} strokeWidth={isActive ? 1.9 : 1.5} />
              Builder
            </>
          )}
        </NavLink>
      </div>
      <div className={itemClassname}>
        <NavLink
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }
          end
          to={`/${projectId}/services/${id}/runner`}
        >
          {({ isActive }) => (
            <>
              <ServerCog height={20} strokeWidth={isActive ? 2 : 1.5} />
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
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }
          end
          to={`/${projectId}/services/${id}/settings`}
        >
          {({ isActive }) => (
            <>
              <Cog height={20} strokeWidth={isActive ? 1.9 : 1.5} />
              Settings
            </>
          )}
        </NavLink>
      </div>
    </CollapsibleWrapper>
  );
};
