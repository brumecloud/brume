import { useFragment } from "@apollo/client";
import { zodResolver } from "@hookform/resolvers/zod";
import { Flame, Loader2, SquareTerminal } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { useBlocker, useNavigate, useParams } from "react-router-dom";
import { toast } from "sonner";
import { z } from "zod";
import type { ServiceFragmentFragment } from "@/_apollo/graphql";
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
import { ServiceFragment } from "@/gql/service.graphql";
import { useDeleteService, useUpdateServiceSettings } from "@/hooks/useService";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

export const SettingPage = () => {
	const { projectId, serviceId } = useParams<RouteParams>();

	const { data: service, complete } = useFragment({
		from: `Service:${serviceId}`,
		fragment: ServiceFragment,
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

	const blocker = useBlocker(() => {
		return form.formState.isDirty;
	});

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
											onClick={submitChanges}
											variant="outline"
											disabled={Object.keys(form.formState.errors).length > 0}
											size="sm"
											className="text-xs"
										>
											Save changes
										</Button>
										<Button
											onClick={() => form.reset()}
											variant="destructive"
											size="sm"
											className="text-xs"
										>
											Discard
										</Button>
									</div>
								)}
							</div>
						</div>
					</div>
				</div>
				<div className="relative flex max-w-[700px] flex-col space-y-4 border-l border-gray-300 pb-16 pl-4">
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
								placeholder="My Service"
								className={cn(
									"w-[400px]",
									fieldState.isDirty && "border-green-500",
								)}
							/>
						)}
					/>
					<p className="text-sm">The universal ID of the service</p>
					<Input value={service?.id} disabled className="w-[400px]" />
				</div>
				<div className="relative flex flex-col border-l border-gray-300 pl-4">
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
				<div className="grow border-l border-gray-300" />
			</div>
		</div>
	);
};
