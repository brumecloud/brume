import { CollapsibleWrapper } from "@/components/collapsable";
import { Skeleton } from "@/components/ui/skeleton";
import { useProjects } from "@/hooks/useProjects";
import type { Project } from "@/schemas/project.schema";
import { modalState } from "@/state/modal.state";
import { cn } from "@/utils";
import {
  BookKey,
  Cpu,
  Gauge,
  NotepadText,
  Plus,
  SquareTerminal,
} from "lucide-react";
import { Link, NavLink } from "react-router-dom";
import { useSnapshot } from "valtio";

import { ServiceMenu } from "./service.menu";

type ProjectViewProps = {
  project: Project;
  isOpen?: boolean;
};

const ProjectView = ({
  project: { name, id, services },
  isOpen,
}: ProjectViewProps) => {
  const itemClassname = cn("flex flex-col gap-1 py-1 rounded");
  return (
    <CollapsibleWrapper title={name} initialIsOpen={isOpen}>
      <div className="flex flex-col gap-1">
        <div className={itemClassname}>
          <NavLink
            to={`/${id}`}
            end
            className={({ isActive }) =>
              cn(
                "flex w-full select-none items-center gap-1 py-1 transition-all hover:cursor-pointer",
                isActive && "font-semibold"
              )
            }>
            {({ isActive }) => (
              <>
                <SquareTerminal
                  strokeWidth={isActive ? 1.9 : 1.5}
                  height={20}
                />
                Overview
              </>
            )}
          </NavLink>
          <div className="flex select-none flex-row items-center gap-1">
            <Cpu strokeWidth={1.5} height={20} />
            Services
          </div>
          <div className="ml-3 flex flex-col gap-y-1 border-l border-gray-200 pl-3 pt-1">
            {services.map((service) => (
              <ServiceMenu
                key={service.id}
                projectId={id}
                service={service}
              />
            ))}
          </div>
        </div>
        <NavLink
          to={`/${id}/variables`}
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }>
          {({ isActive }) => (
            <>
              <BookKey strokeWidth={isActive ? 2 : 1.5} height={20} />
              Variables
            </>
          )}
        </NavLink>
        <NavLink
          to={`/${id}/logs`}
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }>
          {({ isActive }) => (
            <>
              <NotepadText
                strokeWidth={isActive ? 1.9 : 1.5}
                height={20}
              />
              Logs
            </>
          )}
        </NavLink>
        <NavLink
          to={`/${id}/metrics`}
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }>
          {({ isActive }) => (
            <>
              <Gauge strokeWidth={isActive ? 1.9 : 1.5} height={20} />
              Metrics
            </>
          )}
        </NavLink>
      </div>
    </CollapsibleWrapper>
  );
};

export const ProjectMenu = () => {
  const snap = useSnapshot(modalState);
  const { projects, loading } = useProjects();

  const renderLoadingProject = () => {
    return (
      <>
        <Skeleton className="h-[20px] w-full" />
        <Skeleton className="h-[20px] w-full" />
        <Skeleton className="h-[20px] w-full" />
      </>
    );
  };

  const renderProjects = () => {
    return (
      <>
        {projects.map((project, i) => (
          <ProjectView
            key={project.id}
            project={project}
            isOpen={i == 0}
          />
        ))}
      </>
    );
  };

  return (
    <div className="flex flex-col gap-y-3">
      <div className="flex flex-row justify-between">
        <h2 className="select-none text-sm text-gray-400">
          Projects
        </h2>
        <Plus
          className="h-5 w-5 cursor-pointer p-[3px] text-gray-300 transition-all hover:text-gray-500"
          onClick={() => snap.setProjectModalOpen(true)}
        />
      </div>
      {loading && renderLoadingProject()}
      {!loading && renderProjects()}
    </div>
  );
};
