import { AiOutlineCloudServer } from "react-icons/ai";
import { DiGoogleCloudPlatform } from "react-icons/di";
import { FaAws } from "react-icons/fa";
import type { IconType } from "react-icons/lib";
import { VscAzure } from "react-icons/vsc";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { cn } from "@/utils";

const CloudCard = ({
	name,
	icon,
	disabled,
}: {
	name: string;
	icon: IconType;
	disabled?: boolean;
}) => {
	const Icon = icon;
	return (
		<div
			className={cn(
				"w-full flex flex-row items-center justify-between rounded border py-2 px-3",
				disabled && "border-gray-200 cursor-not-allowed",
			)}
		>
			<div className="flex flex-row items-center justify-center gap-4">
				<div className="rounded-full border bg-gray-200 p-2">
					<Icon className={cn("size-5", disabled && "text-gray-400")} />
				</div>
				<div className="flex-col flex">
					<h2
						className={cn(
							"text-sm font-semibold mb-0",
							disabled && "text-gray-400",
						)}
					>
						{name}
					</h2>
					<p
						className={cn(
							"pt-0 mt-0 text-xs text-gray-500",
							disabled && "text-gray-400",
						)}
					>
						Connect any of your {name} account {disabled && "soon"}
					</p>
				</div>
			</div>
			<Link to={`/settings/cloud/${name.toLowerCase()}`}>
				<Button size="sm" variant="outline" disabled={disabled}>
					Connect
				</Button>
			</Link>
		</div>
	);
};

export const CloudsPage = () => {
	return (
		<div className="flex h-full px-32 pt-8">
			<div className="border rounded-sm w-full min-h-32 flex flex-col p-2 gap-8 pb-16">
				{/* header */}
				<div className="flex flex-row justify-center items-center gap-16">
					<div className="flex flex-col items-center gap-2 pt-8">
						<div className="flex flex-col items-center justify-center size-12 rounded border">
							<AiOutlineCloudServer className="size-8" />
						</div>
						<h2 className="text-lg font-medium">Connect your cloud</h2>
						<p className="text-sm text-gray-500">
							Connect one of your cloud account through a highly secure
							connection
						</p>
					</div>
				</div>
				<div className="m-auto max-w-1/2 w-8/12 flex flex-col gap-2">
					<CloudCard name="AWS" icon={FaAws} />
					<CloudCard name="GCP" icon={DiGoogleCloudPlatform} disabled />
					<CloudCard name="Azure" icon={VscAzure} disabled />
				</div>
			</div>
		</div>
	);
};
