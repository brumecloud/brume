import { useEffect } from "react";
import { Link, Outlet, useLocation, useNavigate } from "react-router-dom";
import { Page } from "@/components/page-comp/header";

export const SettingsOutlet = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const path = location.pathname.split("/").pop();

  useEffect(() => {
    if (path === "settings") {
      navigate("account", { replace: true });
    }
  }, [path, navigate]);

  return (
    <Page.Container>
      <Page.Header>
        <Page.Title>Settings</Page.Title>
        <Page.Description>
          Manage your account, all the clouds providers, your domains, etc.
        </Page.Description>
      </Page.Header>
      <Page.Body>
        <div>
          <div className="flex shrink-0 gap-6 border-b">
            <Link
              className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
              data-state={path === "account" ? "active" : ""}
              to="account"
            >
              Account
            </Link>
            <Link
              className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
              data-state={path === "clouds" ? "active" : ""}
              to="clouds"
            >
              Clouds
            </Link>
            <Link
              className="pointer-events-none select-none border-gray-800 py-2 text-gray-400 hover:cursor-not-allowed hover:text-gray-400 data-[state=active]:border-b data-[state=active]:text-gray-800"
              data-state={path === "domains" ? "active" : ""}
              to="domains"
            >
              Domains
            </Link>
          </div>
          <Outlet />
        </div>
      </Page.Body>
    </Page.Container>
  );
};

export const OldSettingsPage = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const path = location.pathname.split("/").pop();

  useEffect(() => {
    if (path === "settings") {
      navigate("account", { replace: true });
    }
  }, [path, navigate]);

  return (
    <div className="flex flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row items-center justify-between pt-16">
          <div className="flex flex-col pb-8">
            <h2 className="pb-2 font-heading text-3xl">Settings</h2>
            <p>
              Manage your account, all the clouds providers, your domains, etc.
            </p>
          </div>
        </div>
        <div className="flex shrink-0 gap-6 border-b">
          <Link
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
            data-state={path === "account" ? "active" : ""}
            to="account"
          >
            Account
          </Link>
          <Link
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
            data-state={path === "clouds" ? "active" : ""}
            to="clouds"
          >
            Clouds
          </Link>
          <Link
            className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
            data-state={path === "domains" ? "active" : ""}
            to="domains"
          >
            Domains
          </Link>
        </div>
      </div>
      <Outlet />
    </div>
  );
};
