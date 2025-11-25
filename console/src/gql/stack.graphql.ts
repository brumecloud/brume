import { gql } from "@/_apollo/gql";

export const DEPLOY_STACK = gql(`
  mutation DeployStack($name: String!, $templateId: String!, $cloudAccountId: String!) {
    deployStack(input: { name: $name, templateId: $templateId, cloudAccountId: $cloudAccountId }) {
      id
    }
  }
`);

export const STACKS_TEMPLATE_FRAGMENT = gql(`
  fragment StacksTemplateFragment on StackTemplate {
    id
    name
  }
`);

export const GET_STACKS_TEMPLATE = gql(`
  query GetStackTemplates {
    getStackTemplates {
      id
      ...StacksTemplateFragment
    }
  }
`);

export const STACK_FRAGMENT = gql(`
  fragment StackFragment on Stack {
    id
    name
    template_id
    status
  }
`);

export const GET_STACKS = gql(`
  query GetStacks {
    getStacks {
      id
      ...StackFragment
    }
  }
`);
