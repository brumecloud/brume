import ProjectFlow from "@/components/flow/project.flow";
import { Button } from "@/components/ui/button";
import { modalState } from "@/state/modal.state";
import { ReactFlowProvider } from "@xyflow/react";
import { useSnapshot } from "valtio";

export function Project() {
  const snap = useSnapshot(modalState);

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-full max-h-[200px] flex-row items-center justify-between px-32 pt-16">
        <div className="flex flex-col pb-8">
          <h2 className="pb-2 font-heading text-3xl">Project Home</h2>
          <p className="">
            This is the home page of this project. You can see an
            overview of the project and the services.
          </p>
        </div>
        <Button onClick={() => snap.setCreateServiceModalOpen(true)}>
          Add a service
        </Button>
      </div>
      <div className="h-full w-full">
        <ReactFlowProvider>
          <ProjectFlow />
        </ReactFlowProvider>
      </div>
    </div>
  );
}
