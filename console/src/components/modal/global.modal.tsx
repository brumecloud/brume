import { DirtyServiceModal } from "@/components/modal/dirty-service/dirty-service.modal";

import { CreateProjectModal } from "./project/createProject.modal";
import { CreateServiceModal } from "./service/createService.modal";

export const GlobalModalRender = () => {
  return (
    <>
      <CreateProjectModal />
      <CreateServiceModal />
      <DirtyServiceModal />
    </>
  );
};
