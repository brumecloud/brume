import { useFragment } from "@apollo/client";
import { Activity } from "lucide-react";
import { MachineFragment } from "@/gql/machine.graphql";

export function MachineCard({ id }: { id: string }) {
	const { data } = useFragment({
		from: `Machine:${id}`,
		fragment: MachineFragment,
	});

	if (!data) {
		return null;
	}

	return (
		<div className="flex h-full w-full flex-col justify-end rounded-lg border bg-card p-6 text-card-foreground shadow-sm">
			<div className="flex flex-col space-y-1.5">
				<div className="flex flex-row items-center justify-between">
					<h3 className="font-regular text-xl leading-none tracking-tight">
						{data.name}
					</h3>
					<div className="flex flex-row items-center justify-center">
						<Activity className="h-4 w-4 stroke-green-300 text-muted-foreground" />
					</div>
				</div>
				<p className="text-sm text-muted-foreground">{data.ip}</p>
			</div>
		</div>
	);
}
