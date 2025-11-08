import { useFragment } from "@apollo/client";
import { zodResolver } from "@hookform/resolvers/zod";
import { ArrowUpFromLine, Code, Loader2, Pickaxe } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Form, useForm } from "react-hook-form";
import { useBlocker, useParams } from "react-router-dom";
import { toast } from "sonner";
import { Label } from "src/components/ui/label";
import { RadioGroup, RadioGroupItem } from "src/components/ui/radio-group";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { useUpdateBuilder } from "@/hooks/useUpdateBuilder";
import {
	DraftBuilderFragment,
	LiveBuilderFragment,
} from "@/router/layout/project.layout";
import type { RouteParams } from "@/router/router.param";
import { type Builder, BuilderSchema } from "@/schemas/service.schema";
import { cn } from "@/utils";

export const BuildTypeValues = {
	GenericImage: "generic-image",
	Dockerfile: "dockerfile",
	StaticWebsite: "static-website",
} as const;

export type BuildType = (typeof BuildTypeValues)[keyof typeof BuildTypeValues];

const _GenericImageOptions = () => {
	const { serviceId } = useParams<RouteParams>();

	const [wasDraft, setWasDraft] = useState(false);

	// TODO : regarder pour passer sur deux useFragment ici au lieu de prendre tout le service
	const { data: draftBuilderData, complete: draftBuilderComplete } =
		useFragment({
			from: `Service:${serviceId}`,
			fragment: DraftBuilderFragment,
			fragmentName: "DraftBuilderFragment",
		});

	const { data: builderData, complete: builderComplete } = useFragment({
		from: `Service:${serviceId}`,
		fragment: LiveBuilderFragment,
		fragmentName: "LiveBuilderFragment",
	});

	if (!draftBuilderComplete) {
		throw new Error("Draft builder not complete");
	}

	if (!builderComplete) {
		throw new Error("Live Builder not complete");
	}

	const draftBuilder = draftBuilderData.draftBuilder;
	const builder = builderData.liveBuilder;

	const { updateBuilderMutation, loading } = useUpdateBuilder();

	const form = useForm<Builder>({
		resolver: zodResolver(BuilderSchema),
		mode: "onChange",
		defaultValues: useMemo(() => {
			if (draftBuilder) {
				return draftBuilder;
			}
			if (builder) {
				return builder;
			}
			return;
		}, [draftBuilder, builder]),
	});

	const blocker = useBlocker(() => form.formState.isDirty);

	if (blocker.state === "blocked" && form.formState.isDirty) {
		toast.warning("You have unsaved changes");
	}

	useEffect(() => {
		if (serviceId) {
			if (draftBuilder) {
				form.reset(draftBuilder);
			} else if (builder) {
				form.reset(builder);
			}
		}
	}, [form, serviceId, builder, draftBuilder]);

	useEffect(() => {
		if (draftBuilder) {
			setWasDraft(true);
		}
		if (wasDraft && !draftBuilder && builder) {
			form.reset(builder);
			setWasDraft(false);
		}
	}, [builder, draftBuilder, form.reset, wasDraft]);

	const onUnload = useCallback(
		(e: BeforeUnloadEvent) => {
			if (form.formState.isDirty) {
				e.preventDefault();
			}
		},
		[form.formState.isDirty],
	);

	useEffect(() => {
		window.addEventListener("beforeunload", onUnload);
		return () => window.removeEventListener("beforeunload", onUnload);
	}, [onUnload]);

	const submitChanges = async () => {
		if (!serviceId) {
			return;
		}

		await updateBuilderMutation({
			variables: {
				serviceId,
				input: form.getValues().data,
			},
		});

		toast.success("Builder updated");
		form.reset(form.getValues());
	};

	if (!(draftBuilder && builder)) {
		return null;
	}

	return (
		<Form {...form}>
			<div className="relative flex flex-col space-y-4 border-gray-300 border-l pb-16 pl-4 transition-all duration-100">
				<div className="flex h-12 flex-row items-center">
					<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
						<Code className="h-5 w-5" />
					</div>
					<div className="flex w-full flex-row items-center justify-between space-x-2 pl-3">
						<div className="font-medium text-sm">Docker image</div>
						{form.formState.isDirty && (
							<div className="flex flex-row items-center space-x-2">
								{loading && <Loader2 className="h-4 w-4 animate-spin" />}
								<Button
									className="text-xs"
									onClick={submitChanges}
									size="sm"
									variant="outline"
								>
									Save changes
								</Button>
								<Button
									className="text-xs"
									onClick={() => form.reset()}
									size="sm"
									variant="destructive"
								>
									Discard
								</Button>
							</div>
						)}
					</div>
				</div>
				<div className="flex flex-col space-y-1">
					<p className="font-medium text-sm">Image information</p>
					<p className="text-sm">
						Choose a registry and provide a valid docker image name.
					</p>
				</div>
				<div className="flex flex-row gap-2">
					<div className="flex flex-col space-y-2">
						<FormField
							control={form.control}
							name="data.registry"
							render={({ field }) => (
								<>
									<Label className="text-xs" htmlFor="registry">
										Registry
									</Label>
									<Select {...field}>
										<SelectTrigger
											className={cn(
												"w-[180px]",
												draftBuilder.data?.registry !==
													builder.data?.registry && "border-blue-500",
												form.formState.dirtyFields.data?.registry &&
													"border-green-500",
											)}
										>
											<SelectValue placeholder="Registry" />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="docker.io">Docker Hub</SelectItem>
											<SelectItem value="ghcr.io">
												GitHub Container Registry
											</SelectItem>
											<SelectItem value="quay.io">Quay</SelectItem>
										</SelectContent>
									</Select>
									{builder &&
										draftBuilder &&
										draftBuilder.data?.registry !== builder.data?.registry && (
											<p className="text-blue-500 text-xs italic">
												old value: {builder.data?.registry}
											</p>
										)}
								</>
							)}
						/>
					</div>
					<div className="flex flex-col space-y-2">
						<FormField
							control={form.control}
							name="data.image"
							render={({ field }) => (
								<>
									<Label className="text-xs" htmlFor="image">
										Image
									</Label>
									<Input
										{...field}
										className={cn(
											builder &&
												draftBuilder &&
												draftBuilder.data?.image !== builder.data?.image &&
												"border-blue-500",
											form.formState.dirtyFields.data?.image &&
												"border-green-500",
										)}
										placeholder="hello-world"
										type="text"
									/>
									{builder &&
										draftBuilder &&
										draftBuilder.data?.image !== builder.data?.image && (
											<p className="text-blue-500 text-xs italic">
												old value: {builder.data?.image}
											</p>
										)}
								</>
							)}
						/>
					</div>
					<div className="flex flex-col space-y-2">
						<FormField
							control={form.control}
							name="data.tag"
							render={({ field }) => (
								<>
									<Label className="text-xs" htmlFor="tag">
										Tag
									</Label>
									<Input
										{...field}
										className={cn(
											builder &&
												draftBuilder &&
												draftBuilder.data?.tag !== builder.data?.tag &&
												"border-blue-500",
											form.formState.dirtyFields.data?.tag &&
												"border-green-500",
										)}
										id="tag"
										placeholder="latest"
										type="text"
									/>
									{builder &&
										draftBuilder &&
										draftBuilder.data?.tag !== builder.data?.tag && (
											<p className="text-blue-500 text-xs italic">
												old value: {builder.data?.tag}
											</p>
										)}
								</>
							)}
						/>
					</div>
				</div>
			</div>
		</Form>
	);
};

const _DockerfileOptions = () => (
	<div className="flex h-full flex-col">
		<div className="relative flex flex-col border-gray-300 border-l pb-16 pl-4">
			<div className="flex flex-row items-center">
				<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
					<Code className="h-5 w-5" />
				</div>
				<div className="pl-4">Source code</div>
			</div>
			<div className="pt-4">test</div>
		</div>
		<div className="relative flex flex-col border-gray-300 border-l pb-16 pl-4">
			<div className="flex flex-row items-center">
				<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
					<Pickaxe className="h-5 w-5" />
				</div>
				<div className="pl-4">Build artifact</div>
			</div>
			<div className="pt-4">test</div>
		</div>
		<div className="relative flex flex-col border-gray-300 border-l pb-4 pl-4">
			<div className="flex flex-row items-center">
				<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
					<ArrowUpFromLine className="h-5 w-5" />
				</div>
				<div className="pl-4">Push Artifact</div>
			</div>
			<div className="pt-4">test</div>
		</div>
	</div>
);

const _StaticWebsiteOptions = () => (
	<div className="relative flex flex-col border-gray-300 border-l pb-16 pl-4">
		<div className="flex flex-row items-center">
			<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
				<Code className="h-5 w-5" />
			</div>
			<div className="pl-4">Static website</div>
		</div>
	</div>
);

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
