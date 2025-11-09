import { gql } from "@/_apollo/gql";

export const RUNNER_FRAGMENT = gql(`
  fragment RunnerFragment on Runner {
    id
    type
    data
    link
    version
    schema
  }
`);
