import { gql } from "@apollo/client";

export const LogFragment = gql`
  fragment LogFragment on Log {
    message
    level
    timestamp
  }
`;

export const LOG_BY_SERVICE_ID = gql`
  ${LogFragment}
  query GetLogPerServiceID($serviceId: String!) {
    serviceLogs(serviceId: $serviceId) {
      ...LogFragment
    }
  }
`;

export const LOG_BY_SERVICE_ID_SUB = gql`
  ${LogFragment}
  subscription GetLogPerServiceIDSub($serviceId: String!) {
    serviceLogs(serviceId: $serviceId) {
      ...LogFragment
    }
  }
`;
