import { useEffect } from "react";
import { Link, Outlet, useLocation, useNavigate } from "react-router-dom";

export const SettingsPage = () => {
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
						<h2 className="font-heading pb-2 text-3xl">Settings</h2>
						<p>
							Manage your account, all the clouds providers, your domains, etc.
						</p>
					</div>
				</div>
				<div className="flex shrink-0 gap-6 border-b">
					<Link
						to="account"
						data-state={path === "account" ? "active" : ""}
						className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
					>
						Account
					</Link>
					<Link
						to="clouds"
						data-state={path === "clouds" ? "active" : ""}
						className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
					>
						Clouds
					</Link>
					<Link
						to="domains"
						data-state={path === "domains" ? "active" : ""}
						className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
					>
						Domains
					</Link>
				</div>
			</div>
			<Outlet />
		</div>
	);
};
