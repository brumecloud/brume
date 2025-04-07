import { Button } from "@/components/ui/button";
import { PROJECT_BY_ID_QUERY } from "@/gql/project.graphql";
import type { RouteParams } from "@/router/router";
import { modalState } from "@/state/modal.state";
import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { useSnapshot } from "valtio";

export const ServicePage = () => {
  const snap = useSnapshot(modalState);
  const { projectId } = useParams<RouteParams>();

  if (!projectId) {
    throw new Error("No project ID found in the URL");
  }

  const { data, loading } = useQuery(PROJECT_BY_ID_QUERY, {
    variables: {
      projectId: projectId,
    },
    fetchPolicy: "cache-first",
  });

  if (loading) {
    return <>loading</>;
  }

  if (!data) {
    throw new Error("No data for the current project ?");
  }

  const { getProjectById } = data;

  return (
    <div>
      Project {getProjectById.name}'s services
      <Button onClick={() => snap.setCreateServiceModalOpen(true)}>
        Add a service
      </Button>
    </div>
  );
};
