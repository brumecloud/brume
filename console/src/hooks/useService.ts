import { PROJECT_BY_ID_QUERY } from "@/gql/project.graphql";
import {
  DELETE_SERVICE_MUTATION,
  UPDATE_SERVICE_SETTINGS_MUTATION,
} from "@/gql/service.graphql";
import type { RouteParams } from "@/router/router";
import { useMutation } from "@apollo/client";
import { useParams } from "react-router-dom";

import { useProject } from "./useProject";

export const useService = () => {
  const { serviceId } = useParams<RouteParams>();
  const { project } = useProject();

  if (!project) {
    return {
      service: null,
    };
  }

  const el = project.services.find(
    (service) => service.id === serviceId
  );

  return {
    service: el,
  };
};

export const useDeleteService = () => {
  const { serviceId, projectId } = useParams<RouteParams>();

  if (!serviceId) {
    throw new Error("Service ID is required");
  }

  const [deleteServiceMutation, { loading, error }] = useMutation(
    DELETE_SERVICE_MUTATION,
    {
      variables: { serviceId },
      update(cache) {
        // we dont update the cache here...
        if (!projectId) {
          console.error(
            `Project ID is required for deleting service ${serviceId}`
          );
          return;
        }

        // get all the projects from the cache
        const rawProject = cache.readQuery({
          query: PROJECT_BY_ID_QUERY,
          variables: { projectId },
        });

        if (!rawProject) {
          console.error(
            `Project with id ${projectId} not found in cache`
          );
          return;
        }

        // remove the deleted service from the project
        const filteredServices =
          rawProject.getProjectById.services.filter(
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
        if (!data) {
          console.error(
            `No data returned from updateServiceSettingsMutation for service ${serviceId}`
          );
          return;
        }

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
