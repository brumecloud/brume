import { MdChecklist, MdPreview } from "react-icons/md";
import { TfiPackage } from "react-icons/tfi";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";

export const DeployStack = () => {
	return (
		<div className="flex flex-col h-full">
			<div className="px-32 pt-8">
				<div className="flex flex-row items-center justify-between pt-16">
					<div className="flex flex-col pb-8">
						<h2 className="font-heading pb-2 text-3xl">Deploy a stack</h2>
						<p>Deploy a new stack on one of your domain</p>
					</div>
				</div>
				<div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4 mt-8">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
							<TfiPackage className="h-5 w-5" />
						</div>
						<div className="pl-4">Choose the domain</div>
					</div>
					<div className="flex flex-col space-y-4 pt-4">
						<div className="flex flex-col space-y-1">
							<div className="text-sm font-medium">All of your domains</div>
							<p className="text-sm text-gray-500">
								Choose on which domain / account you want the stack to be
								deployed on.
							</p>
							<div className="pt-4">
								<Select>
									<SelectTrigger className="w-[300px]">
										<SelectValue placeholder="Select a domain" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="dev">Dev AWS</SelectItem>
										<SelectItem value="prod">Production AWS</SelectItem>
									</SelectContent>
								</Select>
							</div>
						</div>
					</div>
				</div>
				<div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
							<MdPreview className="h-5 w-5" />
						</div>
						<div className="pl-4">Review the changes</div>
					</div>
					<div className="flex flex-col space-y-4 pt-4">
						<div className="flex flex-col space-y-1">
							<div className="text-sm font-medium">Plan of action</div>
							<p className="text-sm text-gray-500">
								All of these ressources will be created on the domain you
								selected.
							</p>
							<div className="pt-4"></div>
						</div>
					</div>
				</div>
				<div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
							<MdChecklist className="h-5 w-5" />
						</div>
						<div className="pl-4">Agreement</div>
					</div>
					<div className="flex flex-col space-y-4 pt-4">
						<div className="flex flex-col space-y-1">
							<div className="text-sm font-medium">Change will be applied</div>
							<p className="text-sm text-gray-500">
								You agree to the term and conditions of the service. You take
								responsibility for the ressources you are deploying.
							</p>
							<div className="pt-4">
								<div className="flex items-center gap-3">
									<Checkbox id="terms" />
									<Label htmlFor="terms">
										Accept the changes and the risks of deploying the stack
									</Label>
								</div>
								<div className="pt-8">
									<Button>Deploy the SPA stack</Button>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
};
