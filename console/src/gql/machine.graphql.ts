import { gql } from "@/_apollo/gql";

export const GET_MACHINES = gql(`
  query GetMachines {
    machine {
      id
      name
      ip
    }
  }
`);
