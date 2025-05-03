import { gql } from "@/_apollo";
import type { RouteParams } from "@/router/router";
import { useQuery } from "@apollo/client";
import React from "react";
import { Outlet, useParams } from "react-router-dom";

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
  const { projectId } = useParams<RouteParams>();

  const { loading } = useQuery(ProjectQuery, {
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
