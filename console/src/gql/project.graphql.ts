import { gql } from "@apollo/client";

export const ProjectFragment = gql`
  fragment ProjectFragment on Project {
    id
    name
    description
    services {
      name
      id
    }
  }
`;

export const CREATE_PROJECT_MUTATION = gql`
  ${ProjectFragment}
  mutation CreateProject($name: String!, $description: String) {
    createProject(name: $name, description: $description) {
      ...ProjectFragment
    }
  }
`;
