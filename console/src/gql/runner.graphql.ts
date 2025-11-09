import { gql } from "@/_apollo/gql";

export const RUNNER_FRAGMENT = gql(`
  fragment RunnerFragment on Runner {
    type
    data
    link
    version
    schema
  }
`);

// export const UPDATE_RUNNER_MUTATION = gql(`
//   mutation UpdateRunner(
//     $serviceId: String!
//     $input: RunnerDataInput!
//   ) {
//     updateRunner(serviceId: $serviceId, data: $input) {
//       ...RunnerFragment
//     }
//   }
// `);
