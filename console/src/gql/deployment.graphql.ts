import { gql } from "@/_apollo/gql";

export const DEPLOYMENT_FRAGMENT = gql(`
  fragment DeploymentFragment on Deployment {
    id
    env
    source {
      type
      branch
      commit
      message
    }
    logs {
      status
      date
      duration
    }
    createdAt
  }
`);
