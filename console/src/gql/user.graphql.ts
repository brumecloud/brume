import { gql } from "@apollo/client";

export const ME_QUERY = gql`
  query me {
    me {
      id
      name
      avatar
    }
  }
`;
