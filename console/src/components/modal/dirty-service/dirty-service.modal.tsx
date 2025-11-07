import { X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { toast } from "sonner";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { useDeleteDraft, useDeploy } from "@/hooks/useDeploy";
import { useProject } from "@/hooks/useProject";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

export const DirtyServiceModal = () => {
  const { projectId } = useParams<RouteParams>();
  const { project } = useProject();
  const { deploy } = useDeploy();
  const { deleteDraft: deleteDraftMutation } = useDeleteDraft(projectId ?? "");
  const [confirmModalOpen, setConfirmModalOpen] = useState(false);
  const [shouldShow, setShouldShow] = useState(false);

  useEffect(() => {
    if (project?.isDirty && !confirmModalOpen) {
      setShouldShow(true);
    } else {
      setShouldShow(false);
    }
  }, [project, confirmModalOpen]);

  const closeModal = () => {
    // we need to remove all the dirty fields
    if (!projectId) {
      throw new Error("Project ID should be defined");
    }

    setShouldShow(false);
    setConfirmModalOpen(true);
  };

  const cancel = () => {
    setConfirmModalOpen(false);
    setShouldShow(true);
  };

  const deleteDraft = async () => {
    const doDeleteDraft = async () => {
      await deleteDraftMutation({ variables: { projectId } });
    };

    setShouldShow(false);
    setConfirmModalOpen(false);

    toast.promise(doDeleteDraft(), {
      loading: "Deleting draft...",
      success: "Draft deleted!",
      error: "Failed to delete draft",
    });
  };

  const submit = async () => {
    const doDeploy = async () => {
      await deploy({ variables: { projectId } });
    };

    setConfirmModalOpen(false);
    setShouldShow(false);

    toast.promise(doDeploy(), {
      loading: "Deploying...",
      success: "Deploy started!",
      error: "Failed to deploy",
    });
  };

  const firstDeploy = project?.services.every((service) => !service.liveRunner);

  return (
    <AnimatePresence>
      <AlertDialog open={confirmModalOpen}>
        <AlertDialogContent className="bg-white">
          <AlertDialogHeader>
            <AlertDialogTitle>Are you sure absolutely sure?</AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone. All the changes will be lost. For
              ever.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={cancel}>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={deleteDraft}>
              Continue
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      {shouldShow && (
        <motion.div
          initial={{ opacity: 0, y: -150 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          exit={{ opacity: 0, y: -150 }}
          className={cn("absolute top-5 left-0 w-screen")}
        >
          <div className="mr-5 flex flex-col items-end">
            <div className="z-[9999] flex items-center justify-between space-x-4 rounded-lg bg-blue-900 px-4 py-2 pr-2 shadow-blue-400/50 shadow-lg">
              {!firstDeploy && (
                <Button variant="link" className="p-0" onClick={closeModal}>
                  <X className="h-4 w-4 text-blue-400" />
                </Button>
              )}
              <h1 className="font-semibold text-blue-400 text-sm">
                {firstDeploy ? "Deploy the project" : "Apply the changes"}
              </h1>
              <Button
                variant="default"
                className="bg-blue-400 text-white shadow-blue-400/50 shadow-md hover:bg-blue-500"
                onClick={submit}
              >
                Deploy
              </Button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
