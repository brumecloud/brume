import { gql } from "@/_apollo";
import type { RouteParams } from "@/router/router";
import { useQuery } from "@apollo/client";
import React from "react";
import { Outlet, useParams } from "react-router-dom";

export const GenericRunnerFragment = gql(`
  fragment GenericRunnerFragment on Runner {
    type
    data {
      command
      healthCheckURL
      memory {
        limit
        request
      }
      cpu {
        limit
        request
      }
      port
      publicDomain
      privateDomain
    }
  }
`);

export const GenericBuilderFragment = gql(`
  fragment GenericBuilderFragment on Builder {
    data {
      image
      registry
      tag
    }
  }
`);

export const DraftBuilderFragment = gql(`
  fragment DraftBuilderFragment on Service {
    draftBuilder {
      ...GenericBuilderFragment @unmask
    }
  }
`);

export const LiveBuilderFragment = gql(`
  fragment LiveBuilderFragment on Service {
    liveBuilder {
      ...GenericBuilderFragment @unmask
    }
  }
`);

export const LiveRunnerFragment = gql(`
  fragment LiveRunnerFragment on Service {
    liveRunner {
      ...GenericRunnerFragment @unmask
    }
  }
`);

export const DraftRunnerFragment = gql(`
  fragment DraftRunnerFragment on Service {
    draftRunner {
      ...GenericRunnerFragment @unmask
    }
  }
`);

export const ProjectQuery = gql(`
  query ProjectQuery($projectId: String!) {
    getProjectById(id: $projectId) {
      ...ProjectFragment
      services {
        ...ServiceFragment
      }
    }
  }
`);

export const ProjectLayout = () => {
  const { projectId } = useParams<RouteParams>() as {
    projectId: string;
  };

  useQuery(ProjectQuery, {
    variables: { projectId },
  });

  // we do not need to render anything here
  // we only data querying
  return (
    <React.Fragment>
      <Outlet />
    </React.Fragment>
  );
};
