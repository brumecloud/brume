import { gql } from "@apollo/client";

export const BUILDER_FRAGMENT = gql`
  fragment BuilderFragment on Builder {
    type
    data {
      tag
      image
      registry
    }
  }
`;

export const UPDATE_BUILDER_MUTATION = gql`
  mutation UpdateBuilder(
    $serviceId: String!
    $input: BuilderDataInput!
  ) {
    updateBuilder(serviceId: $serviceId, data: $input) {
      type
      data {
        tag
        image
        registry
      }
    }
  }
`;
