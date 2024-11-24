import {
  DELETE_DRAFT_MUTATION,
  DEPLOY_PROJECT_MUTATION,
  PROJECT_BY_ID_QUERY,
} from "@/gql/project.graphql";
import { useMutation } from "@apollo/client";

export const useDeploy = () => {
  const [deployProject] = useMutation(DEPLOY_PROJECT_MUTATION);

  return {
    deploy: deployProject,
  };
};

export const useDeleteDraft = (projectId: string) => {
  const [deleteDraft] = useMutation(DELETE_DRAFT_MUTATION, {
    refetchQueries: [
      {
        query: PROJECT_BY_ID_QUERY,
        variables: { projectId },
      },
    ],
  });

  return {
    deleteDraft,
  };
};
