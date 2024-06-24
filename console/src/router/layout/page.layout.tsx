import { Outlet } from "react-router-dom";

export const PageLayout = () => {
  return (
    <div className="h-full w-full overflow-hidden overflow-y-auto rounded-lg border bg-white p-2 pb-16 shadow-sm">
      <Outlet />
    </div>
  );
};
