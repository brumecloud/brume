import { gql } from "@/_apollo/gql";

export const GET_AWS_CLOUD_FORMATION_URL = gql(`
  query GetAWSCloudFormationURL {
    getAWSCloudFormationURL
  }
`);

export const GET_CLOUD_ACCOUNTS = gql(`
  query GetCloudAccounts {
    me {
      organization {
        cloudAccounts {
          ...CloudAccountFragment
        }
      }
    }
  }
`);

export const CLOUD_ACCOUNT_FRAGMENT = gql(`
  fragment CloudAccountFragment on CloudAccount {
    id
    cloudProvider
    name
    status
    stacks {
      id
      name
      template_id
    }
  }
`);

export const CREATE_CLOUD_ACCOUNT = gql(`
  mutation CreateCloudAccount($name: String!, $cloudAccountId: String!, $cloudProvider: CloudProvider!) {
    createCloudAccount(input: {
      name: $name
      accountId: $cloudAccountId
      cloudProvider: $cloudProvider
    }) {
      id
    }
  }
`);
