import { useFragment } from "@apollo/client";
import { zodResolver } from "@hookform/resolvers/zod";
import { Flame, Loader2, SquareTerminal } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { useBlocker, useNavigate, useParams } from "react-router-dom";
import { toast } from "sonner";
import { z } from "zod";
import type { ServiceFragmentFragment } from "@/_apollo/graphql";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ServiceFragment } from "@/gql/service.graphql";
import { useDeleteService, useUpdateServiceSettings } from "@/hooks/useService";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

export const SourcePage = () => {
	const form = useForm<{ name: string; branch: string }>({
		resolver: zodResolver(
			z.object({
				name: z.string(),
				branch: z.string(),
			}),
		),
		mode: "onChange",
		defaultValues: {
			name: "",
			branch: "",
		},
	});

	const blocker = useBlocker(() => form.formState.isDirty);

	if (blocker.state === "blocked") {
		toast.warning("You have unsaved changes");
	}

	return (
		<Page.Container>
			<Page.Header>
				<Page.Title>Sourcing the code</Page.Title>
				<Page.Description>
					Configure the source of code of the service
				</Page.Description>
			</Page.Header>
			<Page.Body className="pt-8">
				<Stepper.Root leftBorder>
					<Stepper.Item>
						<Stepper.Header>
							<Stepper.Icon>
								<SquareTerminal className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Git repository</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{() => (
								<>
									<p className="text-sm">Define the name of the service</p>
									<div>
										<Label className="text-xs" htmlFor="image">
											Repository URL
										</Label>
										<FormField
											control={form.control}
											name="name"
											render={({ field, fieldState }) => (
												<Input
													{...field}
													className={cn(
														"w-[400px] mt-1",
														fieldState.isDirty && "border-green-500",
													)}
													placeholder="github.com/brumecloud/brume"
												/>
											)}
										/>
									</div>
								</>
							)}
						</Stepper.Body>
					</Stepper.Item>
					<Stepper.Item className="h-full">
						<Stepper.Header>
							<Stepper.Icon>
								<SquareTerminal className="h-5 w-5" />
							</Stepper.Icon>
							<Stepper.Title>Trigger configuration</Stepper.Title>
						</Stepper.Header>
						<Stepper.Body>
							{() => (
								<>
									<p className="text-sm">
										Each commit push to this branch will trigger a build of the
										service.
									</p>
									<div>
										<Label className="text-xs" htmlFor="image">
											Branch name
										</Label>
										<FormField
											control={form.control}
											name="branch"
											render={({ field, fieldState }) => (
												<Input
													{...field}
													className={cn(
														"w-[400px] mt-1",
														fieldState.isDirty && "border-green-500",
													)}
													placeholder="main"
												/>
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

export const OldSettingPage = () => {
	const { projectId, serviceId } = useParams<RouteParams>();

	const { data: service, complete } = useFragment({
		from: `Service:${serviceId}`,
		fragment: ServiceFragment,
		fragmentName: "ServiceFragment",
	});

	if (!complete) {
		throw new Error("Service not complete");
	}

	const { deleteServiceMutation } = useDeleteService();
	const [confirmModalOpen, setConfirmModalOpen] = useState(false);
	const { updateServiceSettingsMutation, loading } = useUpdateServiceSettings();
	const navigate = useNavigate();

	const form = useForm<ServiceFragmentFragment>({
		resolver: zodResolver(
			z.object({
				name: z.string(),
			}),
		),
		mode: "onChange",
		defaultValues: service,
	});

	const blocker = useBlocker(() => form.formState.isDirty);

	if (blocker.state === "blocked") {
		toast.warning("You have unsaved changes");
	}

	useEffect(() => {
		if (service) {
			form.reset(service);
		}
	}, [service, form.reset]);

	const onUnload = useCallback(
		(e: BeforeUnloadEvent) => {
			if (form.formState.isDirty) {
				e.preventDefault();
			}
		},
		[form.formState.isDirty],
	);

	const onDelete = useCallback(() => {
		const promise = async () => {
			await deleteServiceMutation();
			navigate(`/${projectId}`);
		};
		toast.promise(promise, {
			loading: "Deleting...",
			success: "Deleted!",
			error: "Failed to delete",
		});
	}, [deleteServiceMutation, projectId, navigate]);

	const cancel = useCallback(() => {
		setConfirmModalOpen(false);
	}, []);

	const submitChanges = async () => {
		const promise = async () => {
			await updateServiceSettingsMutation({
				variables: {
					serviceId: service?.id,
					input: { name: form.getValues().name },
				},
			});
			form.reset();
		};

		toast.promise(promise, {
			loading: "Updating...",
			success: "Updated!",
			error: "Failed to update",
		});
	};

	useEffect(() => {
		window.addEventListener("beforeunload", onUnload);
		return () => window.removeEventListener("beforeunload", onUnload);
	}, [onUnload]);

	return (
		<div className="flex h-full flex-col px-32 pt-8">
			<AlertDialog open={confirmModalOpen}>
				<AlertDialogContent className="bg-white">
					<AlertDialogHeader>
						<AlertDialogTitle>This is danger.</AlertDialogTitle>
						<AlertDialogDescription>
							Once you delete the service, you also lose all the data associated
							to it. This cannot be undone.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel onClick={cancel}>Cancel</AlertDialogCancel>
						<AlertDialogAction
							className="bg-red-700 hover:bg-red-800"
							onClick={onDelete}
						>
							Continue
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
			<div className="flex h-full flex-col pt-16">
				<div className="flex w-full flex-row items-center justify-between pb-16">
					<div>
						<h2 className="pb-2 font-heading text-3xl">Settings</h2>
						<p>Manage the service</p>
					</div>
					<div className="">
						<div className="flex flex-row items-center justify-between">
							<div className="">
								{form.formState.isDirty && (
									<div className="flex flex-row items-center space-x-2">
										{loading && <Loader2 className="h-4 w-4 animate-spin" />}
										<Button
											className="text-xs"
											disabled={Object.keys(form.formState.errors).length > 0}
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
					</div>
				</div>
				<div className="relative flex max-w-[700px] flex-col space-y-4 border-gray-300 border-l pb-16 pl-4">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
							<SquareTerminal className="h-5 w-5" />
						</div>
						<div className="pl-4">General</div>
					</div>
					<p className="text-sm">Define the name of the service</p>
					<FormField
						control={form.control}
						name="name"
						render={({ field, fieldState }) => (
							<Input
								{...field}
								className={cn(
									"w-[400px]",
									fieldState.isDirty && "border-green-500",
								)}
								placeholder="My Service"
							/>
						)}
					/>
					<p className="text-sm">The universal ID of the service</p>
					<Input className="w-[400px]" disabled value={service?.id} />
				</div>
				<div className="relative flex flex-col border-gray-300 border-l pl-4">
					<div className="flex flex-row items-center">
						<div className="absolute left-[-20px] rounded-full border border-red-300 bg-white p-2 text-red-600">
							<Flame className="h-5 w-5" />
						</div>
						<div className="pl-4 text-red-800">Danger zone</div>
					</div>
					<div className="flex flex-col space-y-6 pt-4 text-red-900">
						<div>
							<p>
								Deleting the service will delete all data associated to it :{" "}
								<span className="font-semibold">
									all its artifacts, all its logs and metrics.
								</span>
							</p>
						</div>
						<Button
							className="w-[100px] bg-red-700 hover:bg-red-800"
							onClick={() => setConfirmModalOpen(true)}
						>
							Delete
						</Button>
					</div>
				</div>
				<div className="grow border-gray-300 border-l" />
			</div>
		</div>
	);
};
