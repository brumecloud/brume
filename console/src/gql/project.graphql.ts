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
