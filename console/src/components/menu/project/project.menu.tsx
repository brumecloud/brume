import { CollapsibleWrapper } from "@/components/collapsable";
import { cn } from "@/utils";
import { BookKey, Cog, Cpu, Gauge, Hammer, Network, NotepadText, ServerCog } from "lucide-react";

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
          <div className="ml-3 flex flex-col gap-y-1 border-l border-gray-200 pl-3 pt-1">
            <CollapsibleWrapper title="User API" indent={0}>
              <div className={itemClassname}>
                <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Hammer strokeWidth={1.5} height={20} />
                  Builder
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
                  <Network strokeWidth={1.5} height={20} />
                  Network
                </div>
              </div>
              <div className={itemClassname}>
                <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Cog strokeWidth={1.5} height={20} />
                  Settings
                </div>
              </div>
            </CollapsibleWrapper>
            <CollapsibleWrapper title="Frontend" indent={0}>
              <div className={itemClassname}>
                <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Hammer strokeWidth={1.5} height={20} />
                  Builder
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
                  <Network strokeWidth={1.5} height={20} />
                  Network
                </div>
              </div>
              <div className={itemClassname}>
                <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Cog strokeWidth={1.5} height={20} />
                  Settings
                </div>
              </div>
            </CollapsibleWrapper>
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
      <ProjectView title="Portfolio" isOpen={true} />
      <ProjectView title="GenAI chatbot" isOpen={false} />
    </div>
  );
};
