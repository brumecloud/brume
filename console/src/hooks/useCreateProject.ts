import { CREATE_PROJECT_MUTATION } from "@/gql/project.graphql";
import { ME_QUERY } from "@/gql/user.graphql";
import { useMutation } from "@apollo/client";

export const useCreateProject = () => {
  const [createProjectMutation, { loading, error }] = useMutation(
    CREATE_PROJECT_MUTATION,
    {
      update(cache, { data }) {
        if (!data) {
          return;
        }

        // add the project to the me query (for the navbar)
        const meQuery = cache.readQuery({ query: ME_QUERY });
        if (!meQuery?.me) {
          return;
        }

        const projects = meQuery.me.projects;
        if (!projects) {
          return;
        }

        const updatedProjects = [data.createProject, ...projects];

        cache.writeQuery({
          query: ME_QUERY,
          data: {
            me: {
              ...meQuery.me,
              projects: updatedProjects,
            },
          },
        });
      },
    }
  );

  return {
    createProjectMutation,
    loading,
    error,
  };
};
