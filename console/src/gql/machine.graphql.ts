import { gql } from "@/_apollo/gql";

export const MachineFragment = gql(`
  fragment MachineFragment on Machine {
    id
    name
    ip
  }
`);

export const GET_MACHINES = gql(`
  query GetMachines {
    machine {
      id
      ...MachineFragment
    }
  }
`);
