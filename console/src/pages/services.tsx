import { Button } from "@/components/ui/button";
import { useProject } from "@/hooks/useProject";
import { modalState } from "@/state/modal.state";
import { useSnapshot } from "valtio";

export const ServicePage = () => {
  const snap = useSnapshot(modalState);
  const { project, loading } = useProject();

  if (loading) {
    return <>loading</>;
  }

  return (
    <div>
      Project {project?.name}'s services
      <Button onClick={() => snap.setCreateServiceModalOpen(true)}>
        Add a service
      </Button>
    </div>
  );
};
