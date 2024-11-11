import { gql } from "@apollo/client";

export const RUNNER_FRAGMENT = gql`
  fragment RunnerFragment on Runner {
    type
    data {
      command
      healthCheckURL
      memory {
        limit
        request
      }
      cpu {
        limit
        request
      }
      port
      publicDomain
      privateDomain
    }
  }
`;

export const UPDATE_RUNNER_MUTATION = gql`
  mutation UpdateRunner(
    $serviceId: String!
    $input: RunnerDataInput!
  ) {
    updateRunner(serviceId: $serviceId, data: $input) {
      ...RunnerFragment
    }
  }
  ${RUNNER_FRAGMENT}
`;
