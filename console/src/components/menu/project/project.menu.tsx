import { CollapsibleWrapper } from "@/components/collapsable";
import { cn } from "@/utils";
import {
  BookKey,
  Cog,
  Cpu,
  Gauge,
  Hammer,
  Network,
  NotepadText,
  ServerCog,
  SquareTerminal,
} from "lucide-react";
import { Link } from "react-router-dom";

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
          <Link
            to={`/${title}`}
            className="flex select-none flex-row items-center gap-1 pb-2 hover:cursor-pointer">
            <SquareTerminal strokeWidth={1.5} height={20} />
            Overview
          </Link>
          <Link
            to={`/${title}/services`}
            className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <Cpu strokeWidth={1.5} height={20} />
            Services
          </Link>
          <div className="ml-3 flex flex-col gap-y-1 border-l border-gray-200 pl-3 pt-1">
            <CollapsibleWrapper title="User API" indent={0}>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/user-api/builder`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Hammer strokeWidth={1.5} height={20} />
                  Builder
                </Link>
              </div>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/user-api/runner`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <ServerCog strokeWidth={1.5} height={20} />
                  Runner
                </Link>
              </div>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/user-api/network`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Network strokeWidth={1.5} height={20} />
                  Network
                </Link>
              </div>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/user-api/settings`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Cog strokeWidth={1.5} height={20} />
                  Settings
                </Link>
              </div>
            </CollapsibleWrapper>
            <CollapsibleWrapper title="Frontend" indent={0}>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/frontend/builder`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Hammer strokeWidth={1.5} height={20} />
                  Builder
                </Link>
              </div>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/frontend/runner`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <ServerCog strokeWidth={1.5} height={20} />
                  Runner
                </Link>
              </div>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/frontend/network`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Network strokeWidth={1.5} height={20} />
                  Network
                </Link>
              </div>
              <div className={itemClassname}>
                <Link
                  to={`/${title}/services/frontend/settings`}
                  className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
                  <Cog strokeWidth={1.5} height={20} />
                  Settings
                </Link>
              </div>
            </CollapsibleWrapper>
          </div>
        </div>
        <Link to={`/${title}/variables`} className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <BookKey strokeWidth={1.5} height={20} />
            Variables
          </div>
        </Link>
        <Link to={`/${title}/logs`} className={itemClassname}>
          <div className="flex select-none flex-row items-center gap-1 hover:cursor-pointer">
            <NotepadText strokeWidth={1.5} height={20} />
            Logs
          </div>
        </Link>
        <Link to={`/${title}/metrics`} className={itemClassname}>
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
  return (
    <div className="flex flex-col gap-y-3">
      <h2 className="select-none text-sm text-gray-500">Projects</h2>
      <ProjectView title="Portfolio" isOpen={true} />
      <ProjectView title="GenAI chatbot" isOpen={false} />
    </div>
  );
};
