import { DEPLOY_PROJECT_MUTATION } from "@/gql/project.graphql";
import { useMutation } from "@apollo/client";

export const useDeploy = () => {
  const [deployProject] = useMutation(DEPLOY_PROJECT_MUTATION);

  return {
    deploy: deployProject,
  };
};
