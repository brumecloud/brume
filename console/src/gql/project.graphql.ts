import { gql } from "@/_apollo/gql";

export const PROJECT_FRAGMENT = gql(`
  fragment ProjectFragment on Project {
    id
    name
    description
    isDirty
  }
`);

export const PROJECT_BY_ID_QUERY = gql(`
  query GetProjectById($projectId: String!) {
    getProjectById(id: $projectId) {
      ...ProjectFragment
    }
  }
`);

export const CREATE_PROJECT_MUTATION = gql(`
  mutation CreateProject($name: String!, $description: String) {
    createProject(name: $name, description: $description) {
      ...ProjectFragment
    }
  }
`);

export const DEPLOY_PROJECT_MUTATION = gql(`
  mutation DeployProject($projectId: String!) {
    deployProject(projectId: $projectId) {
      ...ProjectFragment
    }
  }
`);

export const DELETE_DRAFT_MUTATION = gql(`
  mutation DeleteDraft($projectId: String!) {
    deleteDraft(projectId: $projectId) {
      id
    }
  }
`);
