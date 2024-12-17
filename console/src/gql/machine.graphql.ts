import { gql } from "@apollo/client";

export const GET_MACHINES = gql`
  query GetMachines {
    machine {
      id
      name
      ip
    }
  }
`;
