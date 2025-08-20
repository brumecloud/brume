import { useState } from "react";
import { FaArrowRightLong } from "react-icons/fa6";
import { MdChecklist, MdOutlineLocalPolice, MdPreview } from "react-icons/md";
import { TfiPackage } from "react-icons/tfi";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export const AwsPage = () => {
	const [awsAccount, setAwsAccount] = useState<string>("");

	return (
		<Page.Container>
			<Page.Header>
				<Page.Title>Connect your Amazon Web Service account</Page.Title>
				<Page.Description>
					Lets authorize Brume Cloud to operate on your AWS account
				</Page.Description>
			</Page.Header>
			<Page.Body className="h-full pt-16">
				<Stepper.Root>
					<Stepper.Item>
						<Stepper.Header>
							<Stepper.Icon>
								<TfiPackage className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Choose the account</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{({ setStep }) => (
								<>
									<div className="flex flex-col space-y-1">
										<div className="text-sm font-medium">AWS account</div>
										<p className="text-sm text-gray-500">
											Enter the Account ID of the one you want to connect
										</p>
									</div>
									<div className="flex flex-row gap-4">
										<Input
											placeholder="Enter the Account ID (123456789012)"
											className="max-w-96"
											value={awsAccount}
											onChange={(e) => setAwsAccount(e.target.value)}
										/>
										<Button
											className="w-32"
											onClick={() => setStep(1)}
											disabled={awsAccount.length !== 12}
										>
											<FaArrowRightLong className="h-4 w-4" />
											Continue
										</Button>
									</div>
								</>
							)}
						</Stepper.Body>
					</Stepper.Item>
					<Stepper.Item>
						<Stepper.Header>
							<Stepper.Icon>
								<MdChecklist className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Agreement</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{({ setStep }) => {
								return (
									<>
										<div className="flex flex-col space-y-1">
											<div className="text-sm font-medium">
												Brume will operate on your account
											</div>
											<p className="text-sm text-gray-500">
												Brume will operate on your account, only managing its
												ressources. You are responsible for the stack you are
												deploying.
											</p>
										</div>
										<div className="pt-4">
											<div className="flex items-center gap-3">
												<Checkbox
													id="terms"
													onCheckedChange={(checked) => {
														if (checked) {
															setStep(2);
														} else {
															setStep(1);
														}
													}}
												/>
												<Label htmlFor="terms">
													Accept the changes and the risks
												</Label>
											</div>
										</div>
									</>
								);
							}}
						</Stepper.Body>
					</Stepper.Item>
					<Stepper.Item>
						<Stepper.Header>
							<Stepper.Icon>
								<MdOutlineLocalPolice className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Authorizing Brume Cloud</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{({ setStep }) => {
								return (
									<>
										<div className="flex flex-col space-y-1">
											<div className="text-sm font-medium">
												Creating the Assume role
											</div>
											<p className="text-sm text-gray-500">
												Brume will use an assumed role on your account to manage
												the ressources it deploy.
											</p>
										</div>
										<div className="pt-4">
											<Button onClick={() => setStep(3)}>
												Create the role
											</Button>
										</div>
									</>
								);
							}}
						</Stepper.Body>
					</Stepper.Item>
					<Stepper.Item className="h-full">
						<Stepper.Header>
							<Stepper.Icon>
								<MdPreview className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Verification</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{({ setStep }) => {
								return (
									<>
										<div className="flex flex-col space-y-1">
											<div className="text-sm font-medium">
												Deploying the end to end test stack
											</div>
											<p className="text-sm text-gray-500">
												Brume will deploy a very simple stack to verify
												everything is working
											</p>
										</div>
										<div className="pt-4">
											<Button onClick={() => setStep(4)}>
												Start the end to end test
											</Button>
										</div>
									</>
								);
							}}
						</Stepper.Body>
					</Stepper.Item>
				</Stepper.Root>
			</Page.Body>
		</Page.Container>
	);
};

export const OldAwsPage = () => {
	return (
		<div className="flex flex-col h-full">
			<div className="px-32 pt-8">
				<div className="flex flex-row items-center justify-between pt-16">
					<div className="flex flex-col pb-8">
						<h2 className="font-heading pb-2 text-3xl">
							Connect your Amazon Web Service account
						</h2>
						<p>Lets authorize Brume Cloud to operate on your AWS account</p>
					</div>
				</div>
				<div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4 mt-8">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
							<TfiPackage className="h-5 w-5" />
						</div>
						<div className="pl-4">Choose the account</div>
					</div>
					<div className="flex flex-col space-y-4 pt-4">
						<div className="flex flex-col space-y-1">
							<div className="text-sm font-medium">AWS account</div>
							<p className="text-sm text-gray-500">
								Enter the Account ID of the one you want to connect
							</p>
							<div className="pt-4">
								<Input
									placeholder="Enter the Account ID (123456789012)"
									className="max-w-96"
								/>
							</div>
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
							<div className="text-sm font-medium">
								Brume will operate on your account
							</div>
							<p className="text-sm text-gray-500">
								Brume will operate on your account, only managing its
								ressources. You are responsible for the stack you are deploying.
							</p>
							<div className="pt-4">
								<div className="flex items-center gap-3">
									<Checkbox id="terms" />
									<Label htmlFor="terms">
										Accept the changes and the risks
									</Label>
								</div>
							</div>
						</div>
					</div>
				</div>
				<div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
							<MdOutlineLocalPolice className="h-5 w-5" />
						</div>
						<div className="pl-4">Authorizing Brume Cloud</div>
					</div>
					<div className="flex flex-col space-y-4 pt-4">
						<div className="flex flex-col space-y-1">
							<div className="text-sm font-medium">
								Creating the Assume role
							</div>
							<p className="text-sm text-gray-500">
								Brume will use an assumed role on your account to manage the
								ressources it deploy.
							</p>
							<div className="pt-4">
								<Button>Create the role</Button>
							</div>
						</div>
					</div>
				</div>
				<div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
							<MdPreview className="h-5 w-5" />
						</div>
						<div className="pl-4">Verification</div>
					</div>
					<div className="flex flex-col space-y-4 pt-4">
						<div className="flex flex-col space-y-1">
							<div className="text-sm font-medium">
								Deploying the end to end test stack
							</div>
							<p className="text-sm text-gray-500">
								Brume will deploy a very simple stack to verify everything is
								working
							</p>
							<div className="pt-4">
								<Button>Start the end to end test</Button>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
};
