import { proxy } from "valtio";

export const projectState = proxy({
  projectModalOpen: false,
  setProjectModalOpen: (open: boolean) => (projectState.projectModalOpen = open),
});
