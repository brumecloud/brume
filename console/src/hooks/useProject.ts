import { PROJECT_BY_ID_QUERY } from "@/gql/project.graphql";
import type { RouteParams } from "@/router/router";
import {
  ProjectSchema,
  type Project,
} from "@/schemas/project.schema";
import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";

export const useProject = (): {
  project?: Project;
  loading: boolean;
  error?: Error;
} => {
  const { projectId } = useParams<RouteParams>();

  const { data, loading } = useQuery(PROJECT_BY_ID_QUERY, {
    variables: {
      projectId: projectId ?? "no-id-bug",
    },
    // data can come from me query, or need to be fetch when direct link
    fetchPolicy: "cache-first",
  });

  if (loading || !data) {
    return {
      loading: true,
    };
  } else {
    const rawData = ProjectSchema.safeParse(data?.getProjectById);

    if (!rawData.success) {
      throw new Error(rawData.error.message);
    } else {
      return {
        project: rawData.data,
        loading: false,
      };
    }
  }
};
