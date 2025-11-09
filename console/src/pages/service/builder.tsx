import { Pickaxe } from "lucide-react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { Label } from "src/components/ui/label";
import { RadioGroup, RadioGroupItem } from "src/components/ui/radio-group";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { cn } from "@/utils";

export const BuildTypeValues = {
	GenericImage: "generic-image",
	Dockerfile: "dockerfile",
	StaticWebsite: "static-website",
} as const;

export type BuildType = (typeof BuildTypeValues)[keyof typeof BuildTypeValues];

export const BuilderPage = () => {
	const [_buildType, setBuildType] = useState<BuildType>("static-website");
	const form = useForm({});

	return (
		<Page.Container>
			<Page.Header>
				<Page.Title>Building the service</Page.Title>
				<Page.Description>Configure how your service is built</Page.Description>
			</Page.Header>
			<Page.Body className="pt-8">
				<Stepper.Root leftBorder>
					<Stepper.Item>
						<Stepper.Header>
							<Stepper.Icon>
								<Pickaxe className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Artifact type</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{() => (
								<>
									<div className="flex flex-col space-y-1">
										<div className="font-medium text-sm">
											Select the artifact type
										</div>
										<p className="text-gray-500 text-sm">
											Choose the artifact this service will be built into.
										</p>
									</div>
									<RadioGroup
										className="flex flex-col space-y-1"
										defaultValue="static-website"
										onValueChange={(value) => setBuildType(value as BuildType)}
									>
										<div className="flex space-x-2">
											<RadioGroupItem value="static-website" />
											<Label
												className="flex flex-col space-y-1"
												htmlFor="static-website"
											>
												<span className="font-medium">Static Website</span>
												<p className="text-gray-500 text-sm">
													Static HTML/CSS/JavaScript website. No server-side
													rendering.
												</p>
											</Label>
										</div>
									</RadioGroup>
								</>
							)}
						</Stepper.Body>
					</Stepper.Item>
					<Stepper.Item>
						<Stepper.Header>
							<Stepper.Icon>
								<Pickaxe className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Builder selection</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{() => (
								<>
									<div className="flex flex-col space-y-1">
										<div className="font-medium text-sm">Select a builder</div>
										<p className="text-gray-500 text-sm">
											Select the builder you want to use for this service (you
											can always add more builder using the marketplace).
										</p>
									</div>
									<RadioGroup
										className="flex flex-col space-y-1"
										defaultValue="single-page-application-builder"
										onValueChange={(value) => setBuildType(value as BuildType)}
									>
										<div className="flex space-x-2">
											<RadioGroupItem
												defaultChecked
												value="single-page-application-builder"
											/>
											<Label
												className="flex flex-col space-y-1"
												htmlFor="static-website"
											>
												<span className="font-medium">
													Single Page Application Builder
												</span>
												<p className="text-gray-500 text-sm">
													Simple builder using Vercel deploy API to build your
													single page application.
												</p>
											</Label>
										</div>
									</RadioGroup>
								</>
							)}
						</Stepper.Body>
					</Stepper.Item>
					<Stepper.Item className="h-full">
						<Stepper.Header>
							<Stepper.Icon>
								<Pickaxe className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Builder details</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{() => (
								<>
									<div className="flex flex-col space-y-1">
										<div className="font-medium text-sm">
											Configure the builder options
										</div>
										<p className="text-gray-500 text-sm">
											Each builder type has its own options to configure.
										</p>
									</div>
									<div className="flex flex-col gap-1">
										<Label className="text-xs" htmlFor="build-command">
											Root directory
										</Label>
										<FormField
											control={form.control}
											name="data.healthCheckURL"
											render={({ field }) => (
												<>
													<Input
														{...field}
														className={cn("w-[300px]")}
														placeholder="./src"
													/>
												</>
											)}
										/>
									</div>
									<div className="flex flex-col gap-1">
										<Label
											className="text-xs text-foreground/50"
											htmlFor="build-command"
										>
											Build command
										</Label>
										<FormField
											control={form.control}
											name="data.healthCheckURL"
											render={({ field }) => (
												<>
													<Input
														{...field}
														className={cn("w-[300px]")}
														placeholder="vercel build"
													/>
												</>
											)}
										/>
									</div>
									<div className="flex flex-col gap-1">
										<Label
											className="text-xs text-foreground/50"
											htmlFor="build-command"
										>
											Install command
										</Label>
										<FormField
											control={form.control}
											name="data.healthCheckURL"
											render={({ field }) => (
												<>
													<Input
														{...field}
														className={cn("w-[300px]")}
														placeholder="pnpm install"
													/>
												</>
											)}
										/>
									</div>
								</>
							)}
						</Stepper.Body>
					</Stepper.Item>
				</Stepper.Root>
			</Page.Body>
		</Page.Container>
	);
};
