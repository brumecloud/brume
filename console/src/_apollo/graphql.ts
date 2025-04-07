/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
};

export type Builder = {
  __typename?: 'Builder';
  data: BuilderData;
  type: Scalars['String']['output'];
};

export type BuilderData = {
  __typename?: 'BuilderData';
  image: Scalars['String']['output'];
  registry: Scalars['String']['output'];
  tag: Scalars['String']['output'];
};

export type BuilderDataInput = {
  image: Scalars['String']['input'];
  registry: Scalars['String']['input'];
  tag: Scalars['String']['input'];
};

export type CreateServiceInput = {
  image: Scalars['String']['input'];
  name: Scalars['String']['input'];
};

export type Deployment = {
  __typename?: 'Deployment';
  author: User;
  createdAt: Scalars['String']['output'];
  env: Scalars['String']['output'];
  id: Scalars['String']['output'];
  logs: DeploymentLog;
  source: DeploymentSource;
};

export type DeploymentLog = {
  __typename?: 'DeploymentLog';
  date: Scalars['String']['output'];
  duration: Scalars['String']['output'];
  status: Scalars['String']['output'];
};

export type DeploymentSource = {
  __typename?: 'DeploymentSource';
  branch?: Maybe<Scalars['String']['output']>;
  commit?: Maybe<Scalars['String']['output']>;
  message?: Maybe<Scalars['String']['output']>;
  type: Scalars['String']['output'];
};

export type GetLogsInput = {
  filters?: InputMaybe<Array<Scalars['String']['input']>>;
  limit: Scalars['Int']['input'];
  since: Scalars['String']['input'];
};

export type Log = {
  __typename?: 'Log';
  deploymentId: Scalars['String']['output'];
  deploymentName: Scalars['String']['output'];
  level: Scalars['String']['output'];
  message: Scalars['String']['output'];
  serviceId: Scalars['String']['output'];
  timestamp: Scalars['String']['output'];
};

export type Machine = {
  __typename?: 'Machine';
  id: Scalars['String']['output'];
  ip: Scalars['String']['output'];
  name: Scalars['String']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  addServiceToProject: Service;
  createProject: Project;
  deleteDraft: Project;
  deleteService: Service;
  deployProject: Project;
  updateBuilder: Builder;
  updateRunner: Runner;
  updateServiceSettings: Service;
};


export type MutationAddServiceToProjectArgs = {
  input: CreateServiceInput;
  projectId: Scalars['String']['input'];
};


export type MutationCreateProjectArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  name: Scalars['String']['input'];
};


export type MutationDeleteDraftArgs = {
  projectId: Scalars['String']['input'];
};


export type MutationDeleteServiceArgs = {
  serviceId: Scalars['String']['input'];
};


export type MutationDeployProjectArgs = {
  projectId: Scalars['String']['input'];
};


export type MutationUpdateBuilderArgs = {
  data: BuilderDataInput;
  serviceId: Scalars['String']['input'];
};


export type MutationUpdateRunnerArgs = {
  data: RunnerDataInput;
  serviceId: Scalars['String']['input'];
};


export type MutationUpdateServiceSettingsArgs = {
  input: ServiceSettingsInput;
  serviceId: Scalars['String']['input'];
};

export type Project = {
  __typename?: 'Project';
  description: Scalars['String']['output'];
  id: Scalars['String']['output'];
  isDirty: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  services: Array<Service>;
};

export type Query = {
  __typename?: 'Query';
  getProjectById: Project;
  machine: Array<Machine>;
  me: User;
  projectLogs: Array<Maybe<Log>>;
  serviceLogs: Array<Maybe<Log>>;
};


export type QueryGetProjectByIdArgs = {
  id: Scalars['String']['input'];
};


export type QueryProjectLogsArgs = {
  input: GetLogsInput;
  projectId: Scalars['String']['input'];
};


export type QueryServiceLogsArgs = {
  input: GetLogsInput;
  serviceId: Scalars['String']['input'];
};

export type RessourceConstraints = {
  __typename?: 'RessourceConstraints';
  limit: Scalars['Float']['output'];
  request: Scalars['Float']['output'];
};

export type RessourceConstraintsInput = {
  limit: Scalars['Float']['input'];
  request: Scalars['Float']['input'];
};

export type Runner = {
  __typename?: 'Runner';
  data: RunnerData;
  type: Scalars['String']['output'];
};

export type RunnerData = {
  __typename?: 'RunnerData';
  command: Scalars['String']['output'];
  cpu: RessourceConstraints;
  healthCheckURL: Scalars['String']['output'];
  memory: RessourceConstraints;
  port: Scalars['Int']['output'];
  privateDomain: Scalars['String']['output'];
  publicDomain: Scalars['String']['output'];
};

export type RunnerDataInput = {
  command: Scalars['String']['input'];
  cpu: RessourceConstraintsInput;
  healthCheckURL: Scalars['String']['input'];
  memory: RessourceConstraintsInput;
  port: Scalars['Int']['input'];
  privateDomain: Scalars['String']['input'];
  publicDomain: Scalars['String']['input'];
};

export type Service = {
  __typename?: 'Service';
  deployments: Array<Deployment>;
  draftBuilder?: Maybe<Builder>;
  draftRunner?: Maybe<Runner>;
  id: Scalars['String']['output'];
  liveBuilder?: Maybe<Builder>;
  liveRunner?: Maybe<Runner>;
  name: Scalars['String']['output'];
};

export type ServiceEvent = {
  __typename?: 'ServiceEvent';
  data: Scalars['String']['output'];
  id: Scalars['String']['output'];
  serviceId: Scalars['String']['output'];
  timestamp: Scalars['String']['output'];
  type: Scalars['String']['output'];
};

export type ServiceSettingsInput = {
  name: Scalars['String']['input'];
};

export type Subscription = {
  __typename?: 'Subscription';
  projectLogs: Array<Maybe<Log>>;
  serviceEvents: Array<Maybe<ServiceEvent>>;
  serviceLogs: Array<Maybe<Log>>;
};


export type SubscriptionProjectLogsArgs = {
  input: GetLogsInput;
  projectId: Scalars['String']['input'];
};


export type SubscriptionServiceEventsArgs = {
  serviceId: Scalars['String']['input'];
};


export type SubscriptionServiceLogsArgs = {
  input: GetLogsInput;
  serviceId: Scalars['String']['input'];
};

export type User = {
  __typename?: 'User';
  avatar: Scalars['String']['output'];
  id: Scalars['String']['output'];
  name: Scalars['String']['output'];
  projects: Array<Project>;
};

export type BuilderFragmentFragment = { __typename?: 'Builder', type: string, data: { __typename?: 'BuilderData', tag: string, image: string, registry: string } } & { ' $fragmentName'?: 'BuilderFragmentFragment' };

export type UpdateBuilderMutationVariables = Exact<{
  serviceId: Scalars['String']['input'];
  input: BuilderDataInput;
}>;


export type UpdateBuilderMutation = { __typename?: 'Mutation', updateBuilder: (
    { __typename?: 'Builder' }
    & { ' $fragmentRefs'?: { 'BuilderFragmentFragment': BuilderFragmentFragment } }
  ) };

export type DeploymentFragmentFragment = { __typename?: 'Deployment', id: string, env: string, createdAt: string, author: { __typename?: 'User', id: string, name: string, avatar: string }, source: { __typename?: 'DeploymentSource', type: string, branch?: string | null, commit?: string | null, message?: string | null }, logs: { __typename?: 'DeploymentLog', status: string, date: string, duration: string } } & { ' $fragmentName'?: 'DeploymentFragmentFragment' };

export type LogFragmentFragment = { __typename?: 'Log', message: string, level: string, timestamp: string, serviceId: string, deploymentId: string, deploymentName: string } & { ' $fragmentName'?: 'LogFragmentFragment' };

export type GetLogPerProjectIdQueryVariables = Exact<{
  projectId: Scalars['String']['input'];
  since: Scalars['String']['input'];
  limit: Scalars['Int']['input'];
}>;


export type GetLogPerProjectIdQuery = { __typename?: 'Query', projectLogs: Array<(
    { __typename?: 'Log' }
    & { ' $fragmentRefs'?: { 'LogFragmentFragment': LogFragmentFragment } }
  ) | null> };

export type GetLogPerServiceIdSubSubscriptionVariables = Exact<{
  serviceId: Scalars['String']['input'];
  since: Scalars['String']['input'];
  limit: Scalars['Int']['input'];
}>;


export type GetLogPerServiceIdSubSubscription = { __typename?: 'Subscription', serviceLogs: Array<(
    { __typename?: 'Log' }
    & { ' $fragmentRefs'?: { 'LogFragmentFragment': LogFragmentFragment } }
  ) | null> };

export type GetMachinesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMachinesQuery = { __typename?: 'Query', machine: Array<{ __typename?: 'Machine', id: string, name: string, ip: string }> };

export type ProjectFragmentFragment = { __typename?: 'Project', id: string, name: string, description: string, isDirty: boolean, services: Array<{ __typename?: 'Service', name: string, id: string, deployments: Array<(
      { __typename?: 'Deployment' }
      & { ' $fragmentRefs'?: { 'DeploymentFragmentFragment': DeploymentFragmentFragment } }
    )>, liveRunner?: (
      { __typename?: 'Runner' }
      & { ' $fragmentRefs'?: { 'RunnerFragmentFragment': RunnerFragmentFragment } }
    ) | null, draftRunner?: (
      { __typename?: 'Runner' }
      & { ' $fragmentRefs'?: { 'RunnerFragmentFragment': RunnerFragmentFragment } }
    ) | null, liveBuilder?: (
      { __typename?: 'Builder' }
      & { ' $fragmentRefs'?: { 'BuilderFragmentFragment': BuilderFragmentFragment } }
    ) | null, draftBuilder?: (
      { __typename?: 'Builder' }
      & { ' $fragmentRefs'?: { 'BuilderFragmentFragment': BuilderFragmentFragment } }
    ) | null }> } & { ' $fragmentName'?: 'ProjectFragmentFragment' };

export type GetProjectByIdQueryVariables = Exact<{
  projectId: Scalars['String']['input'];
}>;


export type GetProjectByIdQuery = { __typename?: 'Query', getProjectById: (
    { __typename?: 'Project' }
    & { ' $fragmentRefs'?: { 'ProjectFragmentFragment': ProjectFragmentFragment } }
  ) };

export type CreateProjectMutationVariables = Exact<{
  name: Scalars['String']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
}>;


export type CreateProjectMutation = { __typename?: 'Mutation', createProject: (
    { __typename?: 'Project' }
    & { ' $fragmentRefs'?: { 'ProjectFragmentFragment': ProjectFragmentFragment } }
  ) };

export type DeployProjectMutationVariables = Exact<{
  projectId: Scalars['String']['input'];
}>;


export type DeployProjectMutation = { __typename?: 'Mutation', deployProject: (
    { __typename?: 'Project' }
    & { ' $fragmentRefs'?: { 'ProjectFragmentFragment': ProjectFragmentFragment } }
  ) };

export type DeleteDraftMutationVariables = Exact<{
  projectId: Scalars['String']['input'];
}>;


export type DeleteDraftMutation = { __typename?: 'Mutation', deleteDraft: { __typename?: 'Project', id: string } };

export type RunnerFragmentFragment = { __typename?: 'Runner', type: string, data: { __typename?: 'RunnerData', command: string, healthCheckURL: string, port: number, publicDomain: string, privateDomain: string, memory: { __typename?: 'RessourceConstraints', limit: number, request: number }, cpu: { __typename?: 'RessourceConstraints', limit: number, request: number } } } & { ' $fragmentName'?: 'RunnerFragmentFragment' };

export type UpdateRunnerMutationVariables = Exact<{
  serviceId: Scalars['String']['input'];
  input: RunnerDataInput;
}>;


export type UpdateRunnerMutation = { __typename?: 'Mutation', updateRunner: (
    { __typename?: 'Runner' }
    & { ' $fragmentRefs'?: { 'RunnerFragmentFragment': RunnerFragmentFragment } }
  ) };

export type AddServiceToProjectMutationVariables = Exact<{
  projectId: Scalars['String']['input'];
  input: CreateServiceInput;
}>;


export type AddServiceToProjectMutation = { __typename?: 'Mutation', addServiceToProject: { __typename?: 'Service', id: string, name: string } };

export type DeleteServiceMutationVariables = Exact<{
  serviceId: Scalars['String']['input'];
}>;


export type DeleteServiceMutation = { __typename?: 'Mutation', deleteService: { __typename?: 'Service', id: string } };

export type UpdateServiceSettingsMutationVariables = Exact<{
  serviceId: Scalars['String']['input'];
  input: ServiceSettingsInput;
}>;


export type UpdateServiceSettingsMutation = { __typename?: 'Mutation', updateServiceSettings: { __typename?: 'Service', id: string, name: string } };

export type MeQueryVariables = Exact<{ [key: string]: never; }>;


export type MeQuery = { __typename?: 'Query', me: { __typename?: 'User', id: string, name: string, avatar: string, projects: Array<(
      { __typename?: 'Project' }
      & { ' $fragmentRefs'?: { 'ProjectFragmentFragment': ProjectFragmentFragment } }
    )> } };

export type MyProjectsQueryVariables = Exact<{ [key: string]: never; }>;


export type MyProjectsQuery = { __typename?: 'Query', me: { __typename?: 'User', projects: Array<(
      { __typename?: 'Project' }
      & { ' $fragmentRefs'?: { 'ProjectFragmentFragment': ProjectFragmentFragment } }
    )> } };

export const LogFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LogFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Log"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"message"}},{"kind":"Field","name":{"kind":"Name","value":"level"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"serviceId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentName"}}]}}]} as unknown as DocumentNode<LogFragmentFragment, unknown>;
export const DeploymentFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"DeploymentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"env"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}}]}},{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"branch"}},{"kind":"Field","name":{"kind":"Name","value":"commit"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"date"}},{"kind":"Field","name":{"kind":"Name","value":"duration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]} as unknown as DocumentNode<DeploymentFragmentFragment, unknown>;
export const RunnerFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}}]} as unknown as DocumentNode<RunnerFragmentFragment, unknown>;
export const BuilderFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}}]} as unknown as DocumentNode<BuilderFragmentFragment, unknown>;
export const ProjectFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deployments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"DeploymentFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"DeploymentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"env"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}}]}},{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"branch"}},{"kind":"Field","name":{"kind":"Name","value":"commit"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"date"}},{"kind":"Field","name":{"kind":"Name","value":"duration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}}]} as unknown as DocumentNode<ProjectFragmentFragment, unknown>;
export const UpdateBuilderDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateBuilder"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"BuilderDataInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateBuilder"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"serviceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"data"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}}]} as unknown as DocumentNode<UpdateBuilderMutation, UpdateBuilderMutationVariables>;
export const GetLogPerProjectIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLogPerProjectID"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"since"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"limit"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projectLogs"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"projectId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"since"},"value":{"kind":"Variable","name":{"kind":"Name","value":"since"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"limit"},"value":{"kind":"Variable","name":{"kind":"Name","value":"limit"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LogFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LogFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Log"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"message"}},{"kind":"Field","name":{"kind":"Name","value":"level"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"serviceId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentName"}}]}}]} as unknown as DocumentNode<GetLogPerProjectIdQuery, GetLogPerProjectIdQueryVariables>;
export const GetLogPerServiceIdSubDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"GetLogPerServiceIDSub"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"since"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"limit"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"serviceLogs"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"serviceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"since"},"value":{"kind":"Variable","name":{"kind":"Name","value":"since"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"limit"},"value":{"kind":"Variable","name":{"kind":"Name","value":"limit"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LogFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LogFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Log"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"message"}},{"kind":"Field","name":{"kind":"Name","value":"level"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"serviceId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentName"}}]}}]} as unknown as DocumentNode<GetLogPerServiceIdSubSubscription, GetLogPerServiceIdSubSubscriptionVariables>;
export const GetMachinesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetMachines"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"machine"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"ip"}}]}}]}}]} as unknown as DocumentNode<GetMachinesQuery, GetMachinesQueryVariables>;
export const GetProjectByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetProjectById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"getProjectById"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"DeploymentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"env"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}}]}},{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"branch"}},{"kind":"Field","name":{"kind":"Name","value":"commit"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"date"}},{"kind":"Field","name":{"kind":"Name","value":"duration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deployments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"DeploymentFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}}]}}]} as unknown as DocumentNode<GetProjectByIdQuery, GetProjectByIdQueryVariables>;
export const CreateProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"description"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"description"},"value":{"kind":"Variable","name":{"kind":"Name","value":"description"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"DeploymentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"env"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}}]}},{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"branch"}},{"kind":"Field","name":{"kind":"Name","value":"commit"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"date"}},{"kind":"Field","name":{"kind":"Name","value":"duration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deployments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"DeploymentFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}}]}}]} as unknown as DocumentNode<CreateProjectMutation, CreateProjectMutationVariables>;
export const DeployProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeployProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deployProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"projectId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"DeploymentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"env"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}}]}},{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"branch"}},{"kind":"Field","name":{"kind":"Name","value":"commit"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"date"}},{"kind":"Field","name":{"kind":"Name","value":"duration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deployments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"DeploymentFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}}]}}]} as unknown as DocumentNode<DeployProjectMutation, DeployProjectMutationVariables>;
export const DeleteDraftDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteDraft"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteDraft"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"projectId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<DeleteDraftMutation, DeleteDraftMutationVariables>;
export const UpdateRunnerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateRunner"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RunnerDataInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateRunner"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"serviceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"data"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}}]} as unknown as DocumentNode<UpdateRunnerMutation, UpdateRunnerMutationVariables>;
export const AddServiceToProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"AddServiceToProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateServiceInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"addServiceToProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"projectId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<AddServiceToProjectMutation, AddServiceToProjectMutationVariables>;
export const DeleteServiceDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteService"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteService"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"serviceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<DeleteServiceMutation, DeleteServiceMutationVariables>;
export const UpdateServiceSettingsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateServiceSettings"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ServiceSettingsInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateServiceSettings"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"serviceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"serviceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<UpdateServiceSettingsMutation, UpdateServiceSettingsMutationVariables>;
export const MeDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}},{"kind":"Field","name":{"kind":"Name","value":"projects"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"DeploymentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"env"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}}]}},{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"branch"}},{"kind":"Field","name":{"kind":"Name","value":"commit"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"date"}},{"kind":"Field","name":{"kind":"Name","value":"duration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deployments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"DeploymentFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}}]}}]} as unknown as DocumentNode<MeQuery, MeQueryVariables>;
export const MyProjectsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"myProjects"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projects"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"DeploymentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"env"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}}]}},{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"branch"}},{"kind":"Field","name":{"kind":"Name","value":"commit"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"date"}},{"kind":"Field","name":{"kind":"Name","value":"duration"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"command"}},{"kind":"Field","name":{"kind":"Name","value":"healthCheckURL"}},{"kind":"Field","name":{"kind":"Name","value":"memory"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"cpu"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"limit"}},{"kind":"Field","name":{"kind":"Name","value":"request"}}]}},{"kind":"Field","name":{"kind":"Name","value":"port"}},{"kind":"Field","name":{"kind":"Name","value":"publicDomain"}},{"kind":"Field","name":{"kind":"Name","value":"privateDomain"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tag"}},{"kind":"Field","name":{"kind":"Name","value":"image"}},{"kind":"Field","name":{"kind":"Name","value":"registry"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deployments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"DeploymentFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftRunner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"liveBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draftBuilder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}}]}}]} as unknown as DocumentNode<MyProjectsQuery, MyProjectsQueryVariables>;