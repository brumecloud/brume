import ProjectFlow from "@/components/flow/project.flow";
import { Button } from "@/components/ui/button";
import { ReactFlowProvider } from "@xyflow/react";

export function Project() {
  return (
    <div className="flex flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row items-center justify-between pt-16">
          <div className="flex flex-col pb-8">
            <h2 className="pb-2 font-heading text-3xl">
              Project Home
            </h2>
            <p>
              This is the home page of this project. You can see an
              overview of the project and the services.
            </p>
          </div>
        </div>
        <div className="flex h-[500px] w-full flex-row items-center justify-between">
          <ReactFlowProvider>
            <ProjectFlow />
          </ReactFlowProvider>
        </div>
      </div>
    </div>
  );
}
