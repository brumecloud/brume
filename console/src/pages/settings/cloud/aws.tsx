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
	const [agreement, setAgreement] = useState<boolean>(false);

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
													checked={agreement}
													onCheckedChange={(checked) => {
														setAgreement(
															checked === "indeterminate" ? false : checked,
														);
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
