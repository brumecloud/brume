import { gql, useQuery } from "@apollo/client";
import { BookKey, Cpu, Plus, SquareTerminal } from "lucide-react";
import { NavLink } from "react-router-dom";
import { useSnapshot } from "valtio";
import { CollapsibleWrapper } from "@/components/collapsable";
import { Skeleton } from "@/components/ui/skeleton";
import type { Project } from "@/schemas/project.schema";
import { modalState } from "@/state/modal.state";
import { cn } from "@/utils";
import { ServiceMenu } from "./service.menu";

type ProjectViewProps = {
  project: Project;
  isOpen?: boolean;
};

const ProjectView = ({
  project: { name, id, services },
  isOpen,
}: ProjectViewProps) => {
  const itemClassname = cn("flex flex-col gap-1 rounded py-1");
  return (
    <CollapsibleWrapper initialIsOpen={isOpen} title={name}>
      <div className="flex flex-col gap-1">
        <div className={itemClassname}>
          <NavLink
            className={({ isActive }) =>
              cn(
                "flex w-full select-none items-center gap-1 py-1 transition-all hover:cursor-pointer",
                isActive && "font-semibold"
              )
            }
            end
            to={`/${id}`}
          >
            {({ isActive }) => (
              <>
                <SquareTerminal
                  height={20}
                  strokeWidth={isActive ? 1.9 : 1.5}
                />
                Overview
              </>
            )}
          </NavLink>
          <div className="flex select-none flex-row items-center gap-1">
            <Cpu height={20} strokeWidth={1.5} />
            Services
          </div>
          <div className="ml-3 flex flex-col gap-y-1 border-gray-200 border-l pt-1 pl-3">
            {services.map((service) => (
              <ServiceMenu key={service.id} projectId={id} service={service} />
            ))}
          </div>
        </div>
        <NavLink
          className={({ isActive }) =>
            cn(
              "flex select-none flex-row items-center gap-1 hover:cursor-pointer",
              isActive && "font-semibold"
            )
          }
          to={`/${id}/variables`}
        >
          {({ isActive }) => (
            <>
              <BookKey height={20} strokeWidth={isActive ? 2 : 1.5} />
              Variables
            </>
          )}
        </NavLink>
        {/* <NavLink
					to={`/${id}/logs`}
					className={({ isActive }) =>
						cn(
							"flex select-none flex-row items-center gap-1 hover:cursor-pointer",
							isActive && "font-semibold",
						)
					}
				>
					{({ isActive }) => (
						<>
							<NotepadText strokeWidth={isActive ? 1.9 : 1.5} height={20} />
							Logs
						</>
					)}
				</NavLink>
				<NavLink
					to={`/${id}/metrics`}
					className={({ isActive }) =>
						cn(
							"flex select-none flex-row items-center gap-1 hover:cursor-pointer",
							isActive && "font-semibold",
						)
					}
				>
					{({ isActive }) => (
						<>
							<Gauge strokeWidth={isActive ? 1.9 : 1.5} height={20} />
							Metrics
						</>
					)}
				</NavLink> */}
      </div>
    </CollapsibleWrapper>
  );
};

const PROJECTS_QUERY = gql(`
  query myProjects {
    me {
      projects {
        ...ProjectFragment
      }
    }
  }
`);

export const ProjectMenu = () => {
  const snap = useSnapshot(modalState);
  const { data, loading } = useQuery(PROJECTS_QUERY, {});

  const renderLoadingProject = () => (
    <>
      <Skeleton className="h-[20px] w-full" />
      <Skeleton className="h-[20px] w-full" />
      <Skeleton className="h-[20px] w-full" />
    </>
  );

  const renderProjects = () => (
    <>
      {data?.me.projects.map((project, i) => (
        <ProjectView isOpen={i === 0} key={project.id} project={project} />
      ))}
    </>
  );

  return (
    <div className="flex flex-col gap-y-3">
      <div className="flex flex-row justify-between">
        <h2 className="select-none text-gray-400 text-sm">Projects</h2>
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
