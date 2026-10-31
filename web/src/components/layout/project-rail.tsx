import { createLink, useMatchRoute } from "@tanstack/react-router";
import { ScrollTextIcon, WorkflowIcon } from "lucide-react";
import type { ComponentProps } from "react";

import type { ProjectId } from "~/api/types";
import { Tooltip, TooltipPopup, TooltipTrigger } from "~/components/ui/tooltip";

function RailAnchor(props: ComponentProps<"a">) {
  return (
    <a
      {...props}
      className="flex size-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-foreground data-[active=true]:bg-accent data-[active=true]:text-foreground [&_svg]:size-4"
    />
  );
}

const RailLink = createLink(RailAnchor);

interface ProjectRailProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
}

// The column of icons on the left.
export function ProjectRail({ projectId, environmentName }: ProjectRailProps) {
  const matchRoute = useMatchRoute();
  const params = { projectId, env: environmentName };
  // The canvas owns every other page of the environment, service panels included.
  const onLogs = Boolean(matchRoute({ to: "/projects/$projectId/$env/logs", params }));

  return (
    <nav className="flex w-14 shrink-0 flex-col items-center gap-2 pt-2 max-md:hidden">
      <Tooltip>
        <TooltipTrigger
          render={
            <RailLink
              to="/projects/$projectId/$env"
              params={params}
              aria-label="Architecture"
              data-active={!onLogs}
            />
          }
        >
          <WorkflowIcon />
        </TooltipTrigger>
        <TooltipPopup side="right">Architecture</TooltipPopup>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger
          render={
            <RailLink
              to="/projects/$projectId/$env/logs"
              params={params}
              aria-label="Logs"
              data-active={onLogs}
            />
          }
        >
          <ScrollTextIcon />
        </TooltipTrigger>
        <TooltipPopup side="right">Logs</TooltipPopup>
      </Tooltip>
    </nav>
  );
}
