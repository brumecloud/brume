import ArchitectureServiceAmazonCloudFront from "aws-react-icons/icons/ArchitectureServiceAmazonCloudFront";
import ArchitectureServiceAWSPrivateCertificateAuthority from "aws-react-icons/icons/ArchitectureServiceAWSPrivateCertificateAuthority";
import ResourceAmazonSimpleStorageServiceBucket from "aws-react-icons/icons/ResourceAmazonSimpleStorageServiceBucket";
import { FaAws } from "react-icons/fa";
import { LuBadgeCheck } from "react-icons/lu";
import { Input } from "@/components/ui/input";

const StackCard = () => {
	return (
		<div className="border rounded min-w-[400px]">
			<div className="flex flex-row justify-between px-3 items-center border-b h-12">
				<h2>Single Page Application</h2>
				<div className="flex flex-row gap-x-3 justify-center items-center">
					<FaAws className="size-6" />
					<LuBadgeCheck className="size-5" />
				</div>
			</div>
			<div
				className="h-40 bg-gray-50 -z-10 inset-0 w-full 
bg-[radial-gradient(circle,#73737350_1px,transparent_1px)] 
bg-[size:10px_10px] flex justify-center items-center gap-x-2"
			>
				<ResourceAmazonSimpleStorageServiceBucket className="size-10 bg-gray-50" />
				<ArchitectureServiceAmazonCloudFront className="size-10 bg-gray-50 rounded-sm" />
				<ArchitectureServiceAWSPrivateCertificateAuthority className="size-10 bg-gray-50 rounded-sm" />
			</div>
		</div>
	);
};

export const Marketplace = () => {
	return (
		<div className="flex flex-col">
			<div className="px-32 pt-8">
				<div className="flex flex-row items-center justify-between pt-16">
					<div className="flex flex-col pb-8">
						<h2 className="pb-2 font-heading text-3xl">Marketplace</h2>
						<p>
							See all the differents, all battle testes stacks you can deploy
							for your businnes in 2 minutes
						</p>
					</div>
				</div>
				<div className="flex flex-col gap-8">
					<div className="w-full">
						<Input placeholder="Search for a stack" />
					</div>
					<div className="flex flex-row">
						<StackCard />
					</div>
				</div>
			</div>
		</div>
	);
};
