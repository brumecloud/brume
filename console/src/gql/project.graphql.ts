import { gql } from "@apollo/client";

export const ProjectFragment = gql`
  fragment ProjectFragment on User {
    projects {
      id
      name
      description
    }
  }
`;

export const CreateProject = gql`
  mutation CreateProject($name: String!, $description: String) {
    createProject(name: $name, description: $description) {
      id
      name
      description
    }
  }
`;
