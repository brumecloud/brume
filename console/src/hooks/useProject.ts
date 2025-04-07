import { PROJECT_BY_ID_QUERY } from "@/gql/project.graphql";
import type { RouteParams } from "@/router/router";
import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";

export const useProject = () => {
  const { projectId } = useParams<RouteParams>();

  if (!projectId) {
    throw new Error("No project ID found in the URL");
  }

  const { data, loading } = useQuery(PROJECT_BY_ID_QUERY, {
    variables: {
      projectId: projectId,
    },
    // data can come from me query, or need to be fetch when direct link
    fetchPolicy: "cache-first",
  });

  if (loading || !data) {
    return {
      loading: true,
    };
  } else {
    return {
      project: data.getProjectById,
      loading: false,
    };
  }
};
