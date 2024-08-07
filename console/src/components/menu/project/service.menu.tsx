import { CollapsibleWrapper } from "@/components/collapsable";
import type { Service } from "@/schemas/service.schema";
import { cn } from "@/utils";
import { Hammer, ServerCog, Cog, NotepadText } from "lucide-react";
import { Link } from "react-router-dom";

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
        <Link
          to={`/${projectId}/services/${id}/builder`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <Hammer strokeWidth={1.5} height={20} />
          Builder
        </Link>
      </div>
      <div className={itemClassname}>
        <Link
          to={`/${projectId}/services/${id}/runner`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <ServerCog strokeWidth={1.5} height={20} />
          Runner
        </Link>
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
        <Link
          to={`/${projectId}/services/${id}/settings`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <Cog strokeWidth={1.5} height={20} />
          Settings
        </Link>
      </div>
    </CollapsibleWrapper>
  );
};
