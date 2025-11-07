import { Outlet } from "react-router-dom";

export const PageLayout = () => {
  return (
    <div className="h-full w-full overflow-hidden overflow-y-auto rounded-lg border bg-[#fbfbfb] shadow-sm">
      {/* <GlobalModalRender /> */}
      <Outlet />
    </div>
  );
};
