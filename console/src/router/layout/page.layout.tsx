import { GlobalModalRender } from "@/components/modal/global.modal";
import { Outlet } from "react-router-dom";

export const PageLayout = () => {
  return (
    <div className="h-full w-full overflow-hidden overflow-y-auto rounded-lg border bg-white shadow-sm">
      <GlobalModalRender />
      <Outlet />
    </div>
  );
};
