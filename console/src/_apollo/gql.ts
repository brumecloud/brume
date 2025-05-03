/* eslint-disable */
import * as types from './graphql';
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
type Documents = {
    "\n  fragment BuilderFragment on Builder {\n    type\n    data {\n      tag\n      image\n      registry\n    }\n  }\n": typeof types.BuilderFragmentFragmentDoc,
    "\n  mutation UpdateBuilder(\n    $serviceId: String!\n    $input: BuilderDataInput!\n  ) {\n    updateBuilder(serviceId: $serviceId, data: $input) {\n      ...BuilderFragment\n    }\n  }\n": typeof types.UpdateBuilderDocument,
    "\n  fragment DeploymentFragment on Deployment {\n    id\n    env\n    author {\n      id\n      name\n      avatar\n    }\n    source {\n      type\n      branch\n      commit\n      message\n    }\n    logs {\n      status\n      date\n      duration\n    }\n    createdAt\n  }\n": typeof types.DeploymentFragmentFragmentDoc,
    "\n  fragment LogFragment on Log {\n    message\n    level\n    timestamp\n    serviceId\n    deploymentId\n    deploymentName\n  }\n": typeof types.LogFragmentFragmentDoc,
    "\n  query GetLogPerProjectID(\n    $projectId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    projectLogs(\n      projectId: $projectId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n": typeof types.GetLogPerProjectIdDocument,
    "\n  subscription GetLogPerServiceIDSub(\n    $serviceId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    serviceLogs(\n      serviceId: $serviceId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n": typeof types.GetLogPerServiceIdSubDocument,
    "\n  query GetMachines {\n    machine {\n      id\n      name\n      ip\n    }\n  }\n": typeof types.GetMachinesDocument,
    "\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n  }\n": typeof types.ProjectFragmentFragmentDoc,
    "\n  query GetProjectById($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": typeof types.GetProjectByIdDocument,
    "\n  mutation CreateProject($name: String!, $description: String) {\n    createProject(name: $name, description: $description) {\n      ...ProjectFragment\n    }\n  }\n": typeof types.CreateProjectDocument,
    "\n  mutation DeployProject($projectId: String!) {\n    deployProject(projectId: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": typeof types.DeployProjectDocument,
    "\n  mutation DeleteDraft($projectId: String!) {\n    deleteDraft(projectId: $projectId) {\n      id\n    }\n  }\n": typeof types.DeleteDraftDocument,
    "\n  fragment RunnerFragment on Runner {\n    type\n    data {\n      command\n      healthCheckURL\n      memory {\n        limit\n        request\n      }\n      cpu {\n        limit\n        request\n      }\n      port\n      publicDomain\n      privateDomain\n    }\n  }\n": typeof types.RunnerFragmentFragmentDoc,
    "\n  mutation UpdateRunner(\n    $serviceId: String!\n    $input: RunnerDataInput!\n  ) {\n    updateRunner(serviceId: $serviceId, data: $input) {\n      ...RunnerFragment\n    }\n  }\n": typeof types.UpdateRunnerDocument,
    "\n  mutation AddServiceToProject(\n    $projectId: String!\n    $input: CreateServiceInput!\n  ) {\n    addServiceToProject(projectId: $projectId, input: $input) {\n      id\n      name\n    }\n  }\n": typeof types.AddServiceToProjectDocument,
    "\n  mutation DeleteService($serviceId: String!) {\n    deleteService(serviceId: $serviceId) {\n      id\n    }\n  }\n": typeof types.DeleteServiceDocument,
    "\n  mutation UpdateServiceSettings(\n    $serviceId: String!\n    $input: ServiceSettingsInput!\n  ) {\n    updateServiceSettings(serviceId: $serviceId, input: $input) {\n      id\n      name\n    }\n  }\n": typeof types.UpdateServiceSettingsDocument,
    "\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n  }\n": typeof types.UserFragmentFragmentDoc,
    "\n  query me {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n      ...UserFragment\n    }\n  }\n": typeof types.MeDocument,
    "\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n": typeof types.MyProjectsDocument,
    "\n  fragment ServiceFragment on Service {\n      name\n      id\n      deployments {\n        ...DeploymentFragment @unmask\n      }\n      liveRunner {\n        ...RunnerFragment @unmask\n      }\n      draftRunner {\n        ...RunnerFragment @unmask\n      }\n      liveBuilder {\n        ...BuilderFragment @unmask\n      }\n      draftBuilder {\n        ...BuilderFragment @unmask\n      }\n  }\n": typeof types.ServiceFragmentFragmentDoc,
    "\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n": typeof types.ProjectQueryDocument,
};
const documents: Documents = {
    "\n  fragment BuilderFragment on Builder {\n    type\n    data {\n      tag\n      image\n      registry\n    }\n  }\n": types.BuilderFragmentFragmentDoc,
    "\n  mutation UpdateBuilder(\n    $serviceId: String!\n    $input: BuilderDataInput!\n  ) {\n    updateBuilder(serviceId: $serviceId, data: $input) {\n      ...BuilderFragment\n    }\n  }\n": types.UpdateBuilderDocument,
    "\n  fragment DeploymentFragment on Deployment {\n    id\n    env\n    author {\n      id\n      name\n      avatar\n    }\n    source {\n      type\n      branch\n      commit\n      message\n    }\n    logs {\n      status\n      date\n      duration\n    }\n    createdAt\n  }\n": types.DeploymentFragmentFragmentDoc,
    "\n  fragment LogFragment on Log {\n    message\n    level\n    timestamp\n    serviceId\n    deploymentId\n    deploymentName\n  }\n": types.LogFragmentFragmentDoc,
    "\n  query GetLogPerProjectID(\n    $projectId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    projectLogs(\n      projectId: $projectId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n": types.GetLogPerProjectIdDocument,
    "\n  subscription GetLogPerServiceIDSub(\n    $serviceId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    serviceLogs(\n      serviceId: $serviceId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n": types.GetLogPerServiceIdSubDocument,
    "\n  query GetMachines {\n    machine {\n      id\n      name\n      ip\n    }\n  }\n": types.GetMachinesDocument,
    "\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n  }\n": types.ProjectFragmentFragmentDoc,
    "\n  query GetProjectById($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": types.GetProjectByIdDocument,
    "\n  mutation CreateProject($name: String!, $description: String) {\n    createProject(name: $name, description: $description) {\n      ...ProjectFragment\n    }\n  }\n": types.CreateProjectDocument,
    "\n  mutation DeployProject($projectId: String!) {\n    deployProject(projectId: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": types.DeployProjectDocument,
    "\n  mutation DeleteDraft($projectId: String!) {\n    deleteDraft(projectId: $projectId) {\n      id\n    }\n  }\n": types.DeleteDraftDocument,
    "\n  fragment RunnerFragment on Runner {\n    type\n    data {\n      command\n      healthCheckURL\n      memory {\n        limit\n        request\n      }\n      cpu {\n        limit\n        request\n      }\n      port\n      publicDomain\n      privateDomain\n    }\n  }\n": types.RunnerFragmentFragmentDoc,
    "\n  mutation UpdateRunner(\n    $serviceId: String!\n    $input: RunnerDataInput!\n  ) {\n    updateRunner(serviceId: $serviceId, data: $input) {\n      ...RunnerFragment\n    }\n  }\n": types.UpdateRunnerDocument,
    "\n  mutation AddServiceToProject(\n    $projectId: String!\n    $input: CreateServiceInput!\n  ) {\n    addServiceToProject(projectId: $projectId, input: $input) {\n      id\n      name\n    }\n  }\n": types.AddServiceToProjectDocument,
    "\n  mutation DeleteService($serviceId: String!) {\n    deleteService(serviceId: $serviceId) {\n      id\n    }\n  }\n": types.DeleteServiceDocument,
    "\n  mutation UpdateServiceSettings(\n    $serviceId: String!\n    $input: ServiceSettingsInput!\n  ) {\n    updateServiceSettings(serviceId: $serviceId, input: $input) {\n      id\n      name\n    }\n  }\n": types.UpdateServiceSettingsDocument,
    "\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n  }\n": types.UserFragmentFragmentDoc,
    "\n  query me {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n      ...UserFragment\n    }\n  }\n": types.MeDocument,
    "\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n": types.MyProjectsDocument,
    "\n  fragment ServiceFragment on Service {\n      name\n      id\n      deployments {\n        ...DeploymentFragment @unmask\n      }\n      liveRunner {\n        ...RunnerFragment @unmask\n      }\n      draftRunner {\n        ...RunnerFragment @unmask\n      }\n      liveBuilder {\n        ...BuilderFragment @unmask\n      }\n      draftBuilder {\n        ...BuilderFragment @unmask\n      }\n  }\n": types.ServiceFragmentFragmentDoc,
    "\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n": types.ProjectQueryDocument,
};

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = gql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function gql(source: string): unknown;

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment BuilderFragment on Builder {\n    type\n    data {\n      tag\n      image\n      registry\n    }\n  }\n"): (typeof documents)["\n  fragment BuilderFragment on Builder {\n    type\n    data {\n      tag\n      image\n      registry\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation UpdateBuilder(\n    $serviceId: String!\n    $input: BuilderDataInput!\n  ) {\n    updateBuilder(serviceId: $serviceId, data: $input) {\n      ...BuilderFragment\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateBuilder(\n    $serviceId: String!\n    $input: BuilderDataInput!\n  ) {\n    updateBuilder(serviceId: $serviceId, data: $input) {\n      ...BuilderFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment DeploymentFragment on Deployment {\n    id\n    env\n    author {\n      id\n      name\n      avatar\n    }\n    source {\n      type\n      branch\n      commit\n      message\n    }\n    logs {\n      status\n      date\n      duration\n    }\n    createdAt\n  }\n"): (typeof documents)["\n  fragment DeploymentFragment on Deployment {\n    id\n    env\n    author {\n      id\n      name\n      avatar\n    }\n    source {\n      type\n      branch\n      commit\n      message\n    }\n    logs {\n      status\n      date\n      duration\n    }\n    createdAt\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment LogFragment on Log {\n    message\n    level\n    timestamp\n    serviceId\n    deploymentId\n    deploymentName\n  }\n"): (typeof documents)["\n  fragment LogFragment on Log {\n    message\n    level\n    timestamp\n    serviceId\n    deploymentId\n    deploymentName\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query GetLogPerProjectID(\n    $projectId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    projectLogs(\n      projectId: $projectId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n"): (typeof documents)["\n  query GetLogPerProjectID(\n    $projectId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    projectLogs(\n      projectId: $projectId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  subscription GetLogPerServiceIDSub(\n    $serviceId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    serviceLogs(\n      serviceId: $serviceId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n"): (typeof documents)["\n  subscription GetLogPerServiceIDSub(\n    $serviceId: String!\n    $since: String!\n    $limit: Int!\n  ) {\n    serviceLogs(\n      serviceId: $serviceId\n      input: { since: $since, limit: $limit }\n    ) {\n      ...LogFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query GetMachines {\n    machine {\n      id\n      name\n      ip\n    }\n  }\n"): (typeof documents)["\n  query GetMachines {\n    machine {\n      id\n      name\n      ip\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n  }\n"): (typeof documents)["\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query GetProjectById($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n    }\n  }\n"): (typeof documents)["\n  query GetProjectById($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation CreateProject($name: String!, $description: String) {\n    createProject(name: $name, description: $description) {\n      ...ProjectFragment\n    }\n  }\n"): (typeof documents)["\n  mutation CreateProject($name: String!, $description: String) {\n    createProject(name: $name, description: $description) {\n      ...ProjectFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation DeployProject($projectId: String!) {\n    deployProject(projectId: $projectId) {\n      ...ProjectFragment\n    }\n  }\n"): (typeof documents)["\n  mutation DeployProject($projectId: String!) {\n    deployProject(projectId: $projectId) {\n      ...ProjectFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation DeleteDraft($projectId: String!) {\n    deleteDraft(projectId: $projectId) {\n      id\n    }\n  }\n"): (typeof documents)["\n  mutation DeleteDraft($projectId: String!) {\n    deleteDraft(projectId: $projectId) {\n      id\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment RunnerFragment on Runner {\n    type\n    data {\n      command\n      healthCheckURL\n      memory {\n        limit\n        request\n      }\n      cpu {\n        limit\n        request\n      }\n      port\n      publicDomain\n      privateDomain\n    }\n  }\n"): (typeof documents)["\n  fragment RunnerFragment on Runner {\n    type\n    data {\n      command\n      healthCheckURL\n      memory {\n        limit\n        request\n      }\n      cpu {\n        limit\n        request\n      }\n      port\n      publicDomain\n      privateDomain\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation UpdateRunner(\n    $serviceId: String!\n    $input: RunnerDataInput!\n  ) {\n    updateRunner(serviceId: $serviceId, data: $input) {\n      ...RunnerFragment\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateRunner(\n    $serviceId: String!\n    $input: RunnerDataInput!\n  ) {\n    updateRunner(serviceId: $serviceId, data: $input) {\n      ...RunnerFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation AddServiceToProject(\n    $projectId: String!\n    $input: CreateServiceInput!\n  ) {\n    addServiceToProject(projectId: $projectId, input: $input) {\n      id\n      name\n    }\n  }\n"): (typeof documents)["\n  mutation AddServiceToProject(\n    $projectId: String!\n    $input: CreateServiceInput!\n  ) {\n    addServiceToProject(projectId: $projectId, input: $input) {\n      id\n      name\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation DeleteService($serviceId: String!) {\n    deleteService(serviceId: $serviceId) {\n      id\n    }\n  }\n"): (typeof documents)["\n  mutation DeleteService($serviceId: String!) {\n    deleteService(serviceId: $serviceId) {\n      id\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation UpdateServiceSettings(\n    $serviceId: String!\n    $input: ServiceSettingsInput!\n  ) {\n    updateServiceSettings(serviceId: $serviceId, input: $input) {\n      id\n      name\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateServiceSettings(\n    $serviceId: String!\n    $input: ServiceSettingsInput!\n  ) {\n    updateServiceSettings(serviceId: $serviceId, input: $input) {\n      id\n      name\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n  }\n"): (typeof documents)["\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query me {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n      ...UserFragment\n    }\n  }\n"): (typeof documents)["\n  query me {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n      ...UserFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n"): (typeof documents)["\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment ServiceFragment on Service {\n      name\n      id\n      deployments {\n        ...DeploymentFragment @unmask\n      }\n      liveRunner {\n        ...RunnerFragment @unmask\n      }\n      draftRunner {\n        ...RunnerFragment @unmask\n      }\n      liveBuilder {\n        ...BuilderFragment @unmask\n      }\n      draftBuilder {\n        ...BuilderFragment @unmask\n      }\n  }\n"): (typeof documents)["\n  fragment ServiceFragment on Service {\n      name\n      id\n      deployments {\n        ...DeploymentFragment @unmask\n      }\n      liveRunner {\n        ...RunnerFragment @unmask\n      }\n      draftRunner {\n        ...RunnerFragment @unmask\n      }\n      liveBuilder {\n        ...BuilderFragment @unmask\n      }\n      draftBuilder {\n        ...BuilderFragment @unmask\n      }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n"): (typeof documents)["\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n"];

export function gql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;