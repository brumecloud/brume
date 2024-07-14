import { Button } from "@/components/ui/button";
import { modalState } from "@/state/modal.state";
import { useSnapshot } from "valtio";

export const ServicePage = () => {
  const snap = useSnapshot(modalState);

  return (
    <div>
      Service Page
      <Button onClick={() => snap.setCreateServiceModalOpen(true)}>Add a service</Button>
    </div>
  );
};
