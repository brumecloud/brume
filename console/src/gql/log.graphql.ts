// import { gql } from "@/_apollo/gql";

// export const LogFragment = gql(`
//   fragment LogFragment on Log {
//     message
//     level
//     timestamp
//     serviceId
//     deploymentId
//     deploymentName
//   }
// `);

// export const LOG_BY_PROJECT_ID = gql(`
//   query GetLogPerProjectID(
//     $projectId: String!
//     $since: String!
//     $limit: Int!
//   ) {
//     projectLogs(
//       projectId: $projectId
//       input: { since: $since, limit: $limit }
//     ) {
//       ...LogFragment @unmask
//     }
//   }
// `);

// export const LOG_BY_SERVICE_ID_SUB = gql(`
//   subscription GetLogPerServiceIDSub(
//     $serviceId: String!
//     $since: String!
//     $limit: Int!
//   ) {
//     serviceLogs(
//       serviceId: $serviceId
//       input: { since: $since, limit: $limit }
//     ) {
//       ...LogFragment
//     }
//   }
// `);
