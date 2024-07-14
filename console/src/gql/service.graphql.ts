import { gql } from "@apollo/client";

export const ADD_SERVICE_MUTATION = gql`
  mutation AddServiceToProject($projectId: String!, $name: String!) {
    addServiceToProject(projectId: $projectId, name: $name) {
      id
      name
    }
  }
`;
