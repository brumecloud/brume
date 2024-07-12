import { GenerateMenu } from "./general/general.menu";
import { ProjectMenu } from "./project/project.menu";
import { UserMenu } from "./user/user.menu";

export const AsideMenu = () => {
  return (
    <aside className="flex h-full flex-col gap-y-4 p-4 pr-1">
      <GenerateMenu />
      <div className="flex h-full flex-col gap-y-2 overflow-y-auto">
        <ProjectMenu />
      </div>
      <div className="flex-grow" />
      <UserMenu />
    </aside>
  );
};
