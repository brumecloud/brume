import { CollapsibleWrapper } from "@/components/collapsable";
import { cn } from "@/utils";
import { Hammer, ServerCog, Network, Cog } from "lucide-react";
import { Link } from "react-router-dom";

type ServiceViewProps = {
  projectId: string;
  projectName: string;
  serviceId: string;
};

export const ServiceMenu = ({ projectId, projectName, serviceId }: ServiceViewProps) => {
  const itemClassname = cn("flex flex-col gap-1 py-1 rounded");

  return (
    <CollapsibleWrapper title={projectName} indent={0}>
      <div className={itemClassname}>
        <Link
          to={`/${projectId}/services/${serviceId}/builder`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <Hammer strokeWidth={1.5} height={20} />
          Builder
        </Link>
      </div>
      <div className={itemClassname}>
        <Link
          to={`/${projectId}/services/${serviceId}/runner`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <ServerCog strokeWidth={1.5} height={20} />
          Runner
        </Link>
      </div>
      <div className={itemClassname}>
        <Link
          to={`/${projectId}/services/${serviceId}/network`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <Network strokeWidth={1.5} height={20} />
          Network
        </Link>
      </div>
      <div className={itemClassname}>
        <Link
          to={`/${projectId}/services/${serviceId}/settings`}
          className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
          <Cog strokeWidth={1.5} height={20} />
          Settings
        </Link>
      </div>
    </CollapsibleWrapper>
  );
};
