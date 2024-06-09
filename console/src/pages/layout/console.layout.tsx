import { AsideMenu } from "@/components/menu/aside.menu";
import { Outlet } from "react-router-dom";

export const ConsoleLayout = () => {
  return (
    <div className="relative m-0 flex h-screen w-screen justify-between bg-gray-100 p-0">
      <div className="flex h-full w-full flex-row">
        <div className="h-full w-[300px]">
          <AsideMenu />
        </div>
        <div className="h-full w-full p-2">
          <Outlet />
        </div>
      </div>
    </div>
  );
};
