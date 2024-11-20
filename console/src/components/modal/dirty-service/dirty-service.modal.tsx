import { Button } from "@/components/ui/button";
import { useDeploy } from "@/hooks/useDeploy";
import { RouteParams } from "@/router/router";
import { modalState } from "@/state/modal.state";
import { cn } from "@/utils";
import { X } from "lucide-react";
import { useLocation, useParams } from "react-router-dom";
import { toast } from "sonner";
import { useSnapshot } from "valtio";

export const DirtyServiceModal = () => {
  const { projectId } = useParams<RouteParams>();
  const snap = useSnapshot(modalState);
  const { deploy } = useDeploy();

  const closeModal = () => {
    // we need to remove all the dirty fields
    if (!projectId) {
      throw new Error("Project ID should be defined");
    }

    snap.setIsProjectDirty(projectId, false);
  };

  const submit = async () => {
    console.log("deploying", projectId);

    const doDeploy = async () => {
      await deploy({ variables: { projectId } });
      closeModal();
    };

    toast.promise(doDeploy(), {
      loading: "Deploying...",
      success: "Deploy started!",
      error: "Failed to deploy",
    });
  };

  const shouldShow = projectId && snap.isProjectDirty[projectId];

  return (
    <div
      className={cn(
        "absolute left-0 top-5 w-screen",
        shouldShow ? "block" : "hidden"
      )}>
      <div className="flex flex-col items-center">
        <div className="z-[9999] flex items-center justify-between space-x-4 rounded-lg bg-blue-900 px-4 py-2 pr-3 shadow-lg shadow-blue-400/50">
          <Button variant="link" className="p-0" onClick={closeModal}>
            <X className="h-4 w-4 text-blue-400" />
          </Button>
          <h1 className="text-sm font-semibold text-blue-400">
            Apply the changes
          </h1>
          <Button
            variant="outline"
            className="border-blue-200 bg-blue-400 text-white"
            onClick={submit}>
            Deploy
          </Button>
        </div>
      </div>
    </div>
  );
};
