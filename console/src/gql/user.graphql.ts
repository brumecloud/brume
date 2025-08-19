import { gql } from "@/_apollo/gql";

export const USER_FRAGMENT = gql(`
  fragment UserFragment on User {
    id
    name
    avatar
  }
`);

export const ME_QUERY = gql(`
  query me {
    me {
      projects {
        id
        name
        services {
          id
          name
        }
      }
      ...UserFragment
    }
  }
`);
