import { PROJECT_BY_ID_QUERY } from "@/gql/project.graphql";
import {
  DELETE_SERVICE_MUTATION,
  UPDATE_SERVICE_SETTINGS_MUTATION,
} from "@/gql/service.graphql";
import type { RouteParams } from "@/router/router";
import { ProjectSchema } from "@/schemas/project.schema";
import { useMutation } from "@apollo/client";
import { useParams } from "react-router-dom";

import { useProject } from "./useProject";

export const useService = () => {
  const { serviceId } = useParams<RouteParams>();
  const { project } = useProject();

  const el = project?.services.find(
    (service) => service.id === serviceId
  );

  return {
    service: el,
  };
};

export const useDeleteService = (_serviceId?: string) => {
  const { serviceId, projectId } = useParams<RouteParams>();

  if (!_serviceId && !serviceId) {
    throw new Error("Service ID is required");
  }

  const [deleteServiceMutation, { loading, error }] = useMutation(
    DELETE_SERVICE_MUTATION,
    {
      variables: { serviceId },
      update(cache) {
        // we dont update the cache here...
        if (!projectId) return;

        const rawProject = cache.readQuery({
          query: PROJECT_BY_ID_QUERY,
          variables: { projectId },
        });

        console.log(rawProject);

        const { data: project, error } = ProjectSchema.safeParse(
          // @ts-expect-error
          rawProject?.getProjectById
        );

        if (error) {
          console.error(error);
          throw error;
        }

        const filteredServices = project.services.filter(
          (service) => service.id !== serviceId
        );

        cache.modify({
          id: `Project:${projectId}`,
          fields: {
            services: () => filteredServices,
          },
        });
      },
    }
  );

  return {
    deleteServiceMutation,
    loading,
    error,
  };
};

export const useUpdateServiceSettings = () => {
  const { serviceId } = useParams<RouteParams>();

  const [updateServiceSettingsMutation, { loading, error }] =
    useMutation(UPDATE_SERVICE_SETTINGS_MUTATION, {
      update(cache, { data }) {
        cache.modify({
          id: `Service:${serviceId}`,
          fields: {
            name: () => data.updateServiceSettings.name,
          },
        });
      },
    });

  return {
    updateServiceSettingsMutation,
    loading,
    error,
  };
};
