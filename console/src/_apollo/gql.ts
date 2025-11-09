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
    "\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n": typeof types.MyProjectsDocument,
    "\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n    services {\n      ...ServiceFragment\n    }\n  }\n": typeof types.ProjectFragmentFragmentDoc,
    "\n  query GetProjectById($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": typeof types.GetProjectByIdDocument,
    "\n  mutation CreateProject($name: String!, $description: String) {\n    createProject(name: $name, description: $description) {\n      ...ProjectFragment\n    }\n  }\n": typeof types.CreateProjectDocument,
    "\n  mutation DeployProject($projectId: String!) {\n    deployProject(projectId: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": typeof types.DeployProjectDocument,
    "\n  mutation DeleteDraft($projectId: String!) {\n    deleteDraft(projectId: $projectId) {\n      id\n    }\n  }\n": typeof types.DeleteDraftDocument,
    "\n  fragment RunnerFragment on Runner {\n    type\n    data\n    link\n    version\n    schema\n  }\n": typeof types.RunnerFragmentFragmentDoc,
    "\n  mutation AddServiceToProject(\n    $projectId: String!\n    $input: CreateServiceInput!\n  ) {\n    addServiceToProject(projectId: $projectId, input: $input) {\n      id\n      name\n    }\n  }\n": typeof types.AddServiceToProjectDocument,
    "\n  mutation DeleteService($serviceId: String!) {\n    deleteService(serviceId: $serviceId) {\n      id\n    }\n  }\n": typeof types.DeleteServiceDocument,
    "\n  mutation UpdateServiceSettings(\n    $serviceId: String!\n    $input: ServiceSettingsInput!\n  ) {\n    updateServiceSettings(serviceId: $serviceId, input: $input) {\n      id\n      name\n    }\n  }\n": typeof types.UpdateServiceSettingsDocument,
    "\n  fragment BaseServiceFragment on BaseService {\n    source {\n      type\n      data\n    }\n    runner {\n      type\n      link\n      version\n      schema\n      data\n    }\n    builder {\n      type\n      link\n      version\n      schema\n      data\n    }\n  }\n": typeof types.BaseServiceFragmentFragmentDoc,
    "\n  fragment ServiceFragment on Service {\n      name\n      id\n      live {\n        ...BaseServiceFragment\n      }\n      draft {\n        ...BaseServiceFragment\n      }\n   }\n": typeof types.ServiceFragmentFragmentDoc,
    "\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n    organization {\n      id\n      name\n    }\n  }\n": typeof types.UserFragmentFragmentDoc,
    "\n  query me {\n    me {\n      projects {\n        id\n        name\n        services {\n          id\n          name\n        }\n      }\n      ...UserFragment\n    }\n  }\n": typeof types.MeDocument,
    "\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n": typeof types.ProjectQueryDocument,
};
const documents: Documents = {
    "\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n": types.MyProjectsDocument,
    "\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n    services {\n      ...ServiceFragment\n    }\n  }\n": types.ProjectFragmentFragmentDoc,
    "\n  query GetProjectById($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": types.GetProjectByIdDocument,
    "\n  mutation CreateProject($name: String!, $description: String) {\n    createProject(name: $name, description: $description) {\n      ...ProjectFragment\n    }\n  }\n": types.CreateProjectDocument,
    "\n  mutation DeployProject($projectId: String!) {\n    deployProject(projectId: $projectId) {\n      ...ProjectFragment\n    }\n  }\n": types.DeployProjectDocument,
    "\n  mutation DeleteDraft($projectId: String!) {\n    deleteDraft(projectId: $projectId) {\n      id\n    }\n  }\n": types.DeleteDraftDocument,
    "\n  fragment RunnerFragment on Runner {\n    type\n    data\n    link\n    version\n    schema\n  }\n": types.RunnerFragmentFragmentDoc,
    "\n  mutation AddServiceToProject(\n    $projectId: String!\n    $input: CreateServiceInput!\n  ) {\n    addServiceToProject(projectId: $projectId, input: $input) {\n      id\n      name\n    }\n  }\n": types.AddServiceToProjectDocument,
    "\n  mutation DeleteService($serviceId: String!) {\n    deleteService(serviceId: $serviceId) {\n      id\n    }\n  }\n": types.DeleteServiceDocument,
    "\n  mutation UpdateServiceSettings(\n    $serviceId: String!\n    $input: ServiceSettingsInput!\n  ) {\n    updateServiceSettings(serviceId: $serviceId, input: $input) {\n      id\n      name\n    }\n  }\n": types.UpdateServiceSettingsDocument,
    "\n  fragment BaseServiceFragment on BaseService {\n    source {\n      type\n      data\n    }\n    runner {\n      type\n      link\n      version\n      schema\n      data\n    }\n    builder {\n      type\n      link\n      version\n      schema\n      data\n    }\n  }\n": types.BaseServiceFragmentFragmentDoc,
    "\n  fragment ServiceFragment on Service {\n      name\n      id\n      live {\n        ...BaseServiceFragment\n      }\n      draft {\n        ...BaseServiceFragment\n      }\n   }\n": types.ServiceFragmentFragmentDoc,
    "\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n    organization {\n      id\n      name\n    }\n  }\n": types.UserFragmentFragmentDoc,
    "\n  query me {\n    me {\n      projects {\n        id\n        name\n        services {\n          id\n          name\n        }\n      }\n      ...UserFragment\n    }\n  }\n": types.MeDocument,
    "\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n": types.ProjectQueryDocument,
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
export function gql(source: "\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n"): (typeof documents)["\n  query myProjects {\n    me {\n      projects {\n        ...ProjectFragment\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n    services {\n      ...ServiceFragment\n    }\n  }\n"): (typeof documents)["\n  fragment ProjectFragment on Project {\n    id\n    name\n    description\n    isDirty\n    services {\n      ...ServiceFragment\n    }\n  }\n"];
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
export function gql(source: "\n  fragment RunnerFragment on Runner {\n    type\n    data\n    link\n    version\n    schema\n  }\n"): (typeof documents)["\n  fragment RunnerFragment on Runner {\n    type\n    data\n    link\n    version\n    schema\n  }\n"];
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
export function gql(source: "\n  fragment BaseServiceFragment on BaseService {\n    source {\n      type\n      data\n    }\n    runner {\n      type\n      link\n      version\n      schema\n      data\n    }\n    builder {\n      type\n      link\n      version\n      schema\n      data\n    }\n  }\n"): (typeof documents)["\n  fragment BaseServiceFragment on BaseService {\n    source {\n      type\n      data\n    }\n    runner {\n      type\n      link\n      version\n      schema\n      data\n    }\n    builder {\n      type\n      link\n      version\n      schema\n      data\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment ServiceFragment on Service {\n      name\n      id\n      live {\n        ...BaseServiceFragment\n      }\n      draft {\n        ...BaseServiceFragment\n      }\n   }\n"): (typeof documents)["\n  fragment ServiceFragment on Service {\n      name\n      id\n      live {\n        ...BaseServiceFragment\n      }\n      draft {\n        ...BaseServiceFragment\n      }\n   }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n    organization {\n      id\n      name\n    }\n  }\n"): (typeof documents)["\n  fragment UserFragment on User {\n    id\n    name\n    avatar\n    organization {\n      id\n      name\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query me {\n    me {\n      projects {\n        id\n        name\n        services {\n          id\n          name\n        }\n      }\n      ...UserFragment\n    }\n  }\n"): (typeof documents)["\n  query me {\n    me {\n      projects {\n        id\n        name\n        services {\n          id\n          name\n        }\n      }\n      ...UserFragment\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n"): (typeof documents)["\n  query ProjectQuery($projectId: String!) {\n    getProjectById(id: $projectId) {\n      ...ProjectFragment\n      services {\n        ...ServiceFragment\n      }\n    }\n  }\n"];

export function gql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;