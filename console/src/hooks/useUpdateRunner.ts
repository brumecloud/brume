import { UPDATE_RUNNER_MUTATION } from "@/gql/runner.graphql";
import type { RouteParams } from "@/router/router";
import { useMutation } from "@apollo/client";
import { useParams } from "react-router-dom";

export const useUpdateRunner = () => {
  const { serviceId, projectId } = useParams<RouteParams>();

  const [updateRunnerMutation, { loading, error }] = useMutation(
    UPDATE_RUNNER_MUTATION,
    {
      update(cache, { data }) {
        if (!data) {
          console.error(
            `No data returned from updateRunnerMutation for service ${serviceId}`
          );
          return;
        }

        cache.modify({
          id: `Service:${serviceId}`,
          fields: {
            draftRunner: () => data.updateRunner,
          },
        });
        cache.modify({
          id: `Project:${projectId}`,
          fields: {
            isDirty: () => true,
          },
        });
      },
    }
  );

  return {
    updateRunnerMutation,
    loading,
    error,
  };
};
