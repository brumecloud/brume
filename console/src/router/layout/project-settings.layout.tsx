import { Link, Outlet, useLocation } from "react-router-dom";

export const ProjectSettingsLayout = () => {
  const location = useLocation();
  const path = location.pathname.split("/").pop();

  return (
    <div className="flex h-full flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row items-center justify-between pt-16">
          <div className="flex flex-col pb-8">
            <h2 className="pb-2 font-heading text-3xl">Settings</h2>
            <p>General settings about the service</p>
          </div>
        </div>
        <div className="flex shrink-0 gap-6 border-b">
          <Link
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
            data-state={path === "settings" ? "active" : ""}
            to="../settings"
          >
            General
          </Link>
          <Link
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
            data-state={path === "variables" ? "active" : ""}
            to="variables"
          >
            Variables
          </Link>
        </div>
      </div>
      <Outlet />
    </div>
  );
};
