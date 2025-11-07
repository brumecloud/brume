import { useService } from "@/hooks/useService";
import { Link, Outlet, useLocation } from "react-router-dom";

export const RunnerLayout = () => {
  const location = useLocation();
  const { service } = useService();
  const path = location.pathname.split("/").pop();

  return (
    <div className="flex flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row items-center justify-between pt-16">
          <div className="flex flex-col pb-8">
            <h2 className="pb-2 font-heading text-3xl">
              {service?.name} Runner
            </h2>
            <p>Inspect & configure how your code runs</p>
          </div>
        </div>
        <div className="flex shrink-0 gap-6 border-b">
          <Link
            to="../runner"
            data-state={path === "runner" ? "active" : ""}
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
          >
            Overview
          </Link>
          <Link
            to="variables"
            data-state={path === "variables" ? "active" : ""}
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
          >
            Variable
          </Link>
          <Link
            to="deployments"
            data-state={path === "deployments" ? "active" : ""}
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
          >
            Deployments
          </Link>
        </div>
      </div>
      <Outlet />
    </div>
  );
};
