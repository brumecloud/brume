import { gql } from "@/_apollo/gql";

export const ME_QUERY = gql(`
  query me {
    me {
      id
      name
      avatar
      projects {
        ...ProjectFragment
      }
    }
  }
`);
