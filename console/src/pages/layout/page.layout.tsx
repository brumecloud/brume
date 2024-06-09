import { Outlet } from "react-router-dom";

export const PageLayout = () => {
  return (
    <div className="h-full w-full rounded-lg border bg-white p-2 shadow-sm">
      <Outlet />
    </div>
  );
};
