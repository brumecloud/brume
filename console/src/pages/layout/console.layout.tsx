import { AsideMenu } from "@/components/menu/aside.menu";
import { ResizablePanelGroup, ResizablePanel } from "@shadcn/resizable";
import { Outlet } from "react-router-dom";

export const ConsoleLayout = () => {
  return (
    <div className="relative m-0 flex h-screen w-screen justify-between bg-gray-100 p-0">
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel defaultSize={17} className="h-full">
          <AsideMenu />
        </ResizablePanel>
        <ResizablePanel className="h-full p-2">
          <Outlet />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
};
