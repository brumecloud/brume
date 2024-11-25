import { UPDATE_BUILDER_MUTATION } from "@/gql/builder.graphql";
import type { RouteParams } from "@/router/router";
import { useMutation } from "@apollo/client";
import { useParams } from "react-router-dom";

export const useUpdateBuilder = () => {
  const { serviceId, projectId } = useParams<RouteParams>();

  const [updateBuilderMutation, { loading, error }] = useMutation(
    UPDATE_BUILDER_MUTATION,
    {
      update(cache, { data }) {
        cache.modify({
          id: `Service:${serviceId}`,
          fields: {
            draftBuilder: () => data.updateBuilder,
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
    updateBuilderMutation,
    loading,
    error,
  };
};
