import { useFragment } from "@apollo/client";
import { useParams } from "react-router-dom";
import { useSnapshot } from "valtio";
import { Button } from "@/components/ui/button";
import { PROJECT_FRAGMENT } from "@/gql/project.graphql";
import type { RouteParams } from "@/router/router.param";
import { modalState } from "@/state/modal.state";

export const ServicePage = () => {
	const snap = useSnapshot(modalState);
	const { projectId } = useParams<RouteParams>();

	if (!projectId) {
		throw new Error("No project ID found in the URL");
	}

	const { data, complete } = useFragment({
		from: `Project:${projectId}`,
		fragment: PROJECT_FRAGMENT,
	});

	if (!data) {
		throw new Error("No data for the current project ?");
	}

	if (!complete) {
		throw new Error("Project not complete");
	}

	return (
		<div>
			Project {data.name}'s services
			<Button onClick={() => snap.setCreateServiceModalOpen(true)}>
				Add a service
			</Button>
		</div>
	);
};
