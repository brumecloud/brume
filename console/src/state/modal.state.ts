import { proxy } from "valtio";

export const modalState = proxy({
  projectModalOpen: false,
  createServiceModalOpen: false,
  isProjectDirty: {},
  setProjectModalOpen: (open: boolean) =>
    (modalState.projectModalOpen = open),
  setCreateServiceModalOpen: (open: boolean) =>
    (modalState.createServiceModalOpen = open),
  setIsProjectDirty: (projectId: string, open: boolean) => {
    console.log("setIsProjectDirty", projectId, open);
    modalState.isProjectDirty[projectId] = open;
  },
});
