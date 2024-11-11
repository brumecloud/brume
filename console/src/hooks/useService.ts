import type { RouteParams } from "@/router/router";
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
