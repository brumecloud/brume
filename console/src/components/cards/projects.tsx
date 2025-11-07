import { GitHubLogoIcon } from "@radix-ui/react-icons";
import { GitBranchIcon } from "lucide-react";
import { useRef } from "react";

import { AreaChart } from "./area.chart";

type ProjectCardProps = {
  name: string;
  url: string;
};

export function ProjectCard(props: ProjectCardProps) {
  const ref = useRef<HTMLDivElement>(null);

  return (
    <div className="flex flex-col overflow-hidden rounded-md border border-gray-200">
      <div ref={ref} className="h-[160px] w-full bg-red-50">
        <AreaChart
          width={ref.current?.clientWidth || 0}
          height={ref.current?.clientHeight || 0}
        />
      </div>
      <div className="w-full">
        <div className="flex flex-row justify-between p-8">
          <div>
            <h3 className="font-bold">{props.name}</h3>
            <p className="text-gray-500 text-sm italic">{props.url}</p>
          </div>
          <div className="flex flex-col items-end gap-x-2 gap-y-2">
            <p className="mr-1 flex flex-row items-center gap-x-2 text-sm">
              34d ago on
              <span>
                <GitBranchIcon size={20} />
              </span>
              <span className="font-mono">main</span>
            </p>
            <div className="flex flex-row items-center gap-x-2 rounded-full bg-gray-50 px-2 py-1 pr-3">
              <GitHubLogoIcon />
              <p className="text-sm">planchon/user-api</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
