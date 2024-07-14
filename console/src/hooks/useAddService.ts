import { ADD_SERVICE_MUTATION } from "@/gql/service.graphql";
import { useMutation } from "@apollo/client";

export const useAddService = (projectId: string) => {
  const [addServiceMutation, { loading, error }] = useMutation(ADD_SERVICE_MUTATION, {
    update(cache, { data }) {
      cache.modify({
        id: `Project:${projectId}`,
        fields: {
          services: (existing, { toReference }) => {
            console.log("existing", existing, toReference(data.addServiceToProject));
            return [toReference(data.addServiceToProject), ...existing];
          },
        },
      });
    },
  });

  return {
    addServiceMutation,
    loading,
    error,
  };
};
