import { CollapsibleWrapper } from "@/components/collapsable";
import { cn } from "@/utils";
import { BookKey, Cpu, Gauge, Hammer, NotepadText, ServerCog } from "lucide-react";

type ProjectViewProps = {
  title: string;
  isOpen?: boolean;
};

const ProjectView = ({ title, isOpen }: ProjectViewProps) => {
  const itemClassname = cn("flex flex-col gap-1 py-1 rounded");
  return (
    <CollapsibleWrapper title={title} initialIsOpen={isOpen}>
      <div className="flex flex-col gap-1">
        <div className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <Cpu strokeWidth={1.5} height={20} />
            Services
          </div>
          <div className="ml-[11px] flex flex-col gap-y-2 border-l border-gray-200 py-1 pl-4">
            <div className="select-none hover:cursor-pointer">User API</div>
            <div className="select-none hover:cursor-pointer">Frontend</div>
          </div>
        </div>
        <div className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <ServerCog strokeWidth={1.5} height={20} />
            Runner
          </div>
        </div>
        <div className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <Hammer strokeWidth={1.5} height={20} />
            Builder
          </div>
        </div>
        <div className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <BookKey strokeWidth={1.5} height={20} />
            Variables
          </div>
        </div>
        <div className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <NotepadText strokeWidth={1.5} height={20} />
            Logs
          </div>
        </div>
        <div className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <Gauge strokeWidth={1.5} height={20} />
            Metrics
          </div>
        </div>
      </div>
    </CollapsibleWrapper>
  );
};

export const ProjectMenu = () => {
  return (
    <div className="flex flex-col gap-y-3">
      <h2 className="select-none text-sm text-gray-500">Projects</h2>
      <ProjectView title="First project" isOpen={true} />
      <ProjectView title="Second project" isOpen={false} />
    </div>
  );
};
