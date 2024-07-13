import { gql } from "@apollo/client";

import { ProjectFragment } from "./project.graphql";

export const ME_QUERY = gql`
  ${ProjectFragment}
  query me {
    me {
      id
      name
      avatar
      ...ProjectFragment
    }
  }
`;
