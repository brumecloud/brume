import { proxy } from "valtio";

export const modalState = proxy({
  projectModalOpen: false,
  createServiceModalOpen: false,
  setProjectModalOpen: (open: boolean) => (modalState.projectModalOpen = open),
  setCreateServiceModalOpen: (open: boolean) =>
    (modalState.createServiceModalOpen = open),
});
