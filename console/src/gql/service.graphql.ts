import { gql } from "@apollo/client";

export const ADD_SERVICE_MUTATION = gql`
  mutation AddServiceToProject(
    $projectId: String!
    $input: CreateServiceInput!
  ) {
    addServiceToProject(projectId: $projectId, input: $input) {
      id
      name
    }
  }
`;

export const DELETE_SERVICE_MUTATION = gql`
  mutation DeleteService($serviceId: String!) {
    deleteService(serviceId: $serviceId) {
      id
    }
  }
`;
