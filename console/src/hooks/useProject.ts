import { PROJECT_BY_ID_QUERY } from "@/gql/project.graphql";
import type { RouteParams } from "@/router/router";
import {
  ProjectSchema,
  type Project,
} from "@/schemas/project.schema";
import { modalState } from "@/state/modal.state";
import { useQuery } from "@apollo/client";
import { useCallback } from "react";
import { useParams } from "react-router-dom";
import { useSnapshot } from "valtio";

export const useProject = (): {
  project?: Project;
  loading: boolean;
  error?: Error;
  setDirty: (dirty: boolean) => void;
} => {
  const { projectId } = useParams<RouteParams>();
  const snap = useSnapshot(modalState);

  const setDirty = useCallback(
    (dirty: boolean) => {
      snap.setIsProjectDirty(projectId ?? "", dirty);
    },
    [projectId, snap]
  );

  const { data, loading } = useQuery(PROJECT_BY_ID_QUERY, {
    variables: {
      projectId: projectId ?? "test",
    },
    // data can come from me query, or need to be fetch when direct link
    fetchPolicy: "cache-first",
  });

  if (loading || !data) {
    return {
      loading: true,
      setDirty,
    };
  } else {
    const rawData = ProjectSchema.safeParse(data?.getProjectById);

    if (!rawData.success) {
      console.error(rawData.error);
      return {
        project: undefined,
        loading: false,
        error: rawData.error,
        setDirty,
      };
    } else {
      snap.setIsProjectDirty(rawData.data.id, rawData.data.isDirty);

      return {
        project: rawData.data,
        loading: false,
        setDirty,
      };
    }
  }
};
