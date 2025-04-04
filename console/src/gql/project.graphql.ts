import { gql } from "@apollo/client";

import { BUILDER_FRAGMENT } from "./builder.graphql";
import { DEPLOYMENT_FRAGMENT } from "./deployment.graphql";
import { RUNNER_FRAGMENT } from "./runner.graphql";

export const ProjectFragment = gql`
  fragment ProjectFragment on Project {
    id
    name
    description
    isDirty
    services {
      name
      id
      deployments {
        ...DeploymentFragment
      }
      liveRunner {
        ...RunnerFragment
      }
      draftRunner {
        ...RunnerFragment
      }
      liveBuilder {
        ...BuilderFragment
      }
      draftBuilder {
        ...BuilderFragment
      }
    }
  }
  ${RUNNER_FRAGMENT}
  ${BUILDER_FRAGMENT}
  ${DEPLOYMENT_FRAGMENT}
`;

export const PROJECT_BY_ID_QUERY = gql`
  ${ProjectFragment}
  query GetProjectById($projectId: String!) {
    getProjectById(id: $projectId) {
      ...ProjectFragment
    }
  }
`;

export const CREATE_PROJECT_MUTATION = gql`
  ${ProjectFragment}
  mutation CreateProject($name: String!, $description: String) {
    createProject(name: $name, description: $description) {
      ...ProjectFragment
    }
  }
`;

export const DEPLOY_PROJECT_MUTATION = gql`
  ${ProjectFragment}
  mutation DeployProject($projectId: String!) {
    deployProject(projectId: $projectId) {
      ...ProjectFragment
    }
  }
`;

export const DELETE_DRAFT_MUTATION = gql`
  mutation DeleteDraft($projectId: String!) {
    deleteDraft(projectId: $projectId) {
      id
    }
  }
`;
