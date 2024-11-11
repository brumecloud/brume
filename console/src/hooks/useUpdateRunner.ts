import { UPDATE_RUNNER_MUTATION } from "@/gql/runner.graphql";
import type { RouteParams } from "@/router/router";
import { useMutation } from "@apollo/client";
import { useParams } from "react-router-dom";

export const useUpdateRunner = () => {
  const { serviceId } = useParams<RouteParams>();

  const [updateRunnerMutation, { loading, error }] = useMutation(
    UPDATE_RUNNER_MUTATION,
    {
      update(cache, { data }) {
        cache.modify({
          id: `Service:${serviceId}`,
          fields: {
            builder: () => data.updateBuilder,
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
