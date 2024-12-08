import { gql } from "@apollo/client";

export const LogFragment = gql`
  fragment LogFragment on Log {
    message
    level
    timestamp
    deploymentId
    deploymentName
  }
`;

export const LOG_BY_PROJECT_ID = gql`
  ${LogFragment}
  query GetLogPerProjectID($projectId: String!) {
    projectLogs(projectId: $projectId) {
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
