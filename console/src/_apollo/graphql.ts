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
  Any: { input: any; output: any; }
};

export type BaseService = {
  __typename?: 'BaseService';
  builder: Builder;
  runner: Runner;
  source: Source;
};

export type Builder = {
  __typename?: 'Builder';
  data: Scalars['Any']['output'];
  id: Scalars['String']['output'];
  link: Scalars['String']['output'];
  schema: Scalars['Any']['output'];
  type: Scalars['String']['output'];
  version: Scalars['String']['output'];
};

export type CloudAccount = {
  __typename?: 'CloudAccount';
  cloudProvider: CloudProvider;
  id: Scalars['String']['output'];
  name: Scalars['String']['output'];
  stacks: Array<Stack>;
  status: CloudStatus;
};

export enum CloudProvider {
  Aws = 'AWS',
  Azure = 'Azure',
  Gcp = 'GCP'
}

export enum CloudStatus {
  Connected = 'Connected',
  Disconnected = 'Disconnected',
  Error = 'Error',
  Pending = 'Pending'
}

export type CreateCloudAccountInput = {
  accountId: Scalars['String']['input'];
  cloudProvider: CloudProvider;
  name: Scalars['String']['input'];
};

export type CreateCloudAccountResponse = {
  __typename?: 'CreateCloudAccountResponse';
  id: Scalars['String']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  createCloudAccount: CreateCloudAccountResponse;
};


export type MutationCreateCloudAccountArgs = {
  input: CreateCloudAccountInput;
};

export type Organization = {
  __typename?: 'Organization';
  cloudAccounts: Array<CloudAccount>;
  id: Scalars['String']['output'];
  name: Scalars['String']['output'];
  providerId: Scalars['String']['output'];
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
  getAWSCloudFormationURL: Scalars['String']['output'];
  getProjectById: Project;
  me: User;
};


export type QueryGetProjectByIdArgs = {
  id: Scalars['String']['input'];
};

export type Runner = {
  __typename?: 'Runner';
  data: Scalars['Any']['output'];
  id: Scalars['String']['output'];
  link: Scalars['String']['output'];
  schema: Scalars['Any']['output'];
  type: Scalars['String']['output'];
  version: Scalars['String']['output'];
};

export type Service = {
  __typename?: 'Service';
  draft?: Maybe<BaseService>;
  id: Scalars['String']['output'];
  live: BaseService;
  name: Scalars['String']['output'];
};

export type Source = {
  __typename?: 'Source';
  data: Scalars['Any']['output'];
  id: Scalars['String']['output'];
  type: Scalars['String']['output'];
};

export type Stack = {
  __typename?: 'Stack';
  id: Scalars['String']['output'];
  name: Scalars['String']['output'];
  template_id: Scalars['String']['output'];
};

export type User = {
  __typename?: 'User';
  avatar: Scalars['String']['output'];
  id: Scalars['String']['output'];
  name: Scalars['String']['output'];
  organization: Organization;
  projects: Array<Project>;
};

export type MyProjectsQueryVariables = Exact<{ [key: string]: never; }>;


export type MyProjectsQuery = { __typename?: 'Query', me: { __typename?: 'User', projects: Array<{ __typename?: 'Project', id: string, name: string, description: string, isDirty: boolean, services: Array<{ __typename?: 'Service', name: string, id: string, live: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } }, draft?: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } } | null }> }> } };

export type BuilderFragmentFragment = { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any };

export type GetAwsCloudFormationUrlQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAwsCloudFormationUrlQuery = { __typename?: 'Query', getAWSCloudFormationURL: string };

export type GetCloudAccountsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetCloudAccountsQuery = { __typename?: 'Query', me: { __typename?: 'User', organization: { __typename?: 'Organization', cloudAccounts: Array<{ __typename?: 'CloudAccount', id: string, cloudProvider: CloudProvider, name: string, status: CloudStatus, stacks: Array<{ __typename?: 'Stack', id: string, name: string, template_id: string }> }> } } };

export type CloudAccountFragmentFragment = { __typename?: 'CloudAccount', id: string, cloudProvider: CloudProvider, name: string, status: CloudStatus, stacks: Array<{ __typename?: 'Stack', id: string, name: string, template_id: string }> };

export type CreateCloudAccountMutationVariables = Exact<{
  name: Scalars['String']['input'];
  cloudAccountId: Scalars['String']['input'];
  cloudProvider: CloudProvider;
}>;


export type CreateCloudAccountMutation = { __typename?: 'Mutation', createCloudAccount: { __typename?: 'CreateCloudAccountResponse', id: string } };

export type ProjectFragmentFragment = { __typename?: 'Project', id: string, name: string, description: string, isDirty: boolean, services: Array<{ __typename?: 'Service', name: string, id: string, live: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } }, draft?: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } } | null }> };

export type GetProjectByIdQueryVariables = Exact<{
  projectId: Scalars['String']['input'];
}>;


export type GetProjectByIdQuery = { __typename?: 'Query', getProjectById: { __typename?: 'Project', id: string, name: string, description: string, isDirty: boolean, services: Array<{ __typename?: 'Service', name: string, id: string, live: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } }, draft?: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } } | null }> } };

export type RunnerFragmentFragment = { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any };

export type BaseServiceFragmentFragment = { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } };

export type ServiceFragmentFragment = { __typename?: 'Service', name: string, id: string, live: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } }, draft?: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } } | null };

export type SourceFragmentFragment = { __typename?: 'Source', id: string, type: string, data: any };

export type UserFragmentFragment = { __typename?: 'User', id: string, name: string, avatar: string, organization: { __typename?: 'Organization', id: string, name: string } };

export type MeQueryVariables = Exact<{ [key: string]: never; }>;


export type MeQuery = { __typename?: 'Query', me: { __typename?: 'User', id: string, name: string, avatar: string, projects: Array<{ __typename?: 'Project', id: string, name: string, services: Array<{ __typename?: 'Service', id: string, name: string }> }>, organization: { __typename?: 'Organization', id: string, name: string } } };

export type ProjectQueryQueryVariables = Exact<{
  projectId: Scalars['String']['input'];
}>;


export type ProjectQueryQuery = { __typename?: 'Query', getProjectById: { __typename?: 'Project', id: string, name: string, description: string, isDirty: boolean, services: Array<{ __typename?: 'Service', name: string, id: string, live: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } }, draft?: { __typename?: 'BaseService', source: { __typename?: 'Source', id: string, type: string, data: any }, runner: { __typename?: 'Runner', id: string, type: string, data: any, link: string, version: string, schema: any }, builder: { __typename?: 'Builder', id: string, type: string, data: any, link: string, version: string, schema: any } } | null }> } };

export const CloudAccountFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CloudAccountFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CloudAccount"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"cloudProvider"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"stacks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"template_id"}}]}}]}}]} as unknown as DocumentNode<CloudAccountFragmentFragment, unknown>;
export const SourceFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SourceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Source"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}}]}}]} as unknown as DocumentNode<SourceFragmentFragment, unknown>;
export const RunnerFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}}]} as unknown as DocumentNode<RunnerFragmentFragment, unknown>;
export const BuilderFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}}]} as unknown as DocumentNode<BuilderFragmentFragment, unknown>;
export const BaseServiceFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BaseServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BaseService"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SourceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"runner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"builder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SourceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Source"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}}]} as unknown as DocumentNode<BaseServiceFragmentFragment, unknown>;
export const ServiceFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Service"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"live"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draft"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SourceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Source"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BaseServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BaseService"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SourceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"runner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"builder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}}]} as unknown as DocumentNode<ServiceFragmentFragment, unknown>;
export const ProjectFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ServiceFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SourceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Source"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BaseServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BaseService"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SourceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"runner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"builder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Service"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"live"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draft"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}}]}}]} as unknown as DocumentNode<ProjectFragmentFragment, unknown>;
export const UserFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}},{"kind":"Field","name":{"kind":"Name","value":"organization"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<UserFragmentFragment, unknown>;
export const MyProjectsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"myProjects"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projects"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SourceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Source"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BaseServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BaseService"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SourceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"runner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"builder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Service"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"live"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draft"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ServiceFragment"}}]}}]}}]} as unknown as DocumentNode<MyProjectsQuery, MyProjectsQueryVariables>;
export const GetAwsCloudFormationUrlDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetAWSCloudFormationURL"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"getAWSCloudFormationURL"}}]}}]} as unknown as DocumentNode<GetAwsCloudFormationUrlQuery, GetAwsCloudFormationUrlQueryVariables>;
export const GetCloudAccountsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetCloudAccounts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"organization"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cloudAccounts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"CloudAccountFragment"}}]}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CloudAccountFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CloudAccount"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"cloudProvider"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"stacks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"template_id"}}]}}]}}]} as unknown as DocumentNode<GetCloudAccountsQuery, GetCloudAccountsQueryVariables>;
export const CreateCloudAccountDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateCloudAccount"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"cloudAccountId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"cloudProvider"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CloudProvider"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createCloudAccount"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"accountId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"cloudAccountId"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"cloudProvider"},"value":{"kind":"Variable","name":{"kind":"Name","value":"cloudProvider"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<CreateCloudAccountMutation, CreateCloudAccountMutationVariables>;
export const GetProjectByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetProjectById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"getProjectById"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SourceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Source"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BaseServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BaseService"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SourceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"runner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"builder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Service"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"live"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draft"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ServiceFragment"}}]}}]}}]} as unknown as DocumentNode<GetProjectByIdQuery, GetProjectByIdQueryVariables>;
export const MeDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projects"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"avatar"}},{"kind":"Field","name":{"kind":"Name","value":"organization"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<MeQuery, MeQueryVariables>;
export const ProjectQueryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ProjectQuery"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"getProjectById"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProjectFragment"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ServiceFragment"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SourceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Source"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"RunnerFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Runner"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BuilderFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Builder"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"type"}},{"kind":"Field","name":{"kind":"Name","value":"data"}},{"kind":"Field","name":{"kind":"Name","value":"link"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"schema"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"BaseServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"BaseService"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"source"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SourceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"runner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"RunnerFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"builder"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BuilderFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ServiceFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Service"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"live"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"draft"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"BaseServiceFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProjectFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"isDirty"}},{"kind":"Field","name":{"kind":"Name","value":"services"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ServiceFragment"}}]}}]}}]} as unknown as DocumentNode<ProjectQueryQuery, ProjectQueryQueryVariables>;