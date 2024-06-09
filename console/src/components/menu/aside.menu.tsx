import { ComputeMenu } from "./compute/compute.menu";
import { GenerateMenu } from "./general/general.menu";
import { NetworkMenu } from "./network/network.menu";
import { ProjectMenu } from "./project/project.menu";
import { UserMenu } from "./user/user.menu";

export const AsideMenu = () => {
  return (
    <aside className="flex h-full flex-col gap-y-4 p-4">
      <GenerateMenu />
      <div className="flex h-full flex-col gap-y-2 overflow-y-auto">
        <ProjectMenu />
        <ComputeMenu />
        <NetworkMenu />
      </div>
      <div className="flex-grow" />
      <UserMenu />
    </aside>
  );
};
