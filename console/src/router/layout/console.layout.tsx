import { useEffect } from "react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import bg from "@/assets/bg.svg";
import { AsideMenu } from "@/components/menu/aside.menu";

export const ConsoleLayout = () => {
  const location = useLocation();
  const nav = useNavigate();

  useEffect(() => {
    if (location.pathname === "/") {
      nav("/overview", { replace: true });
    }
  }, [location, nav]);

  return (
    <div
      className="relative m-0 flex h-screen w-screen justify-between bg-cover bg-gray-50 bg-fixed bg-no-repeat p-0"
      style={{
        backgroundImage: `url(${bg})`,
      }}
    >
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
