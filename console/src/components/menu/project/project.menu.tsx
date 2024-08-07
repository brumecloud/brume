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
import { Link } from "react-router-dom";
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
          <Link
            to={`/${id}`}
            className="flex select-none flex-row items-center gap-1 pb-2 hover:cursor-pointer">
            <SquareTerminal strokeWidth={1.5} height={20} />
            Overview
          </Link>
          <Link
            to={`/${id}/services`}
            className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <Cpu strokeWidth={1.5} height={20} />
            Services
          </Link>
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
        <Link to={`/${id}/variables`} className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <BookKey strokeWidth={1.5} height={20} />
            Variables
          </div>
        </Link>
        <Link to={`/${id}/logs`} className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <NotepadText strokeWidth={1.5} height={20} />
            Logs
          </div>
        </Link>
        <Link to={`/${id}/metrics`} className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <Gauge strokeWidth={1.5} height={20} />
            Metrics
          </div>
        </Link>
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
