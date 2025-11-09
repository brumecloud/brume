import { useQuery } from "@apollo/client";
import { Outlet, useParams } from "react-router-dom";
import { gql } from "@/_apollo/gql";
import type { RouteParams } from "@/router/router.param";

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

	const { loading } = useQuery(ProjectQuery, {
		variables: { projectId },
	});

	if (loading) {
		return <div>Loading...</div>;
	}

	// we do not need to render anything here
	// we only data querying
	return <Outlet />;
};
