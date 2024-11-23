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

export const useDeleteDraft = () => {
  const [deleteDraft] = useMutation(DELETE_DRAFT_MUTATION, {
    update(cache, { data }) {
      console.log(data);
      cache.evict({ id: `Project:${data.deleteDraft.id}` });
    },
  });

  return {
    deleteDraft,
  };
};
