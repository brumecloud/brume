import { CREATE_PROJECT_MUTATION } from "@/gql/project.graphql";
import { ME_QUERY } from "@/gql/user.graphql";
import {
  type Project,
  ProjectSchema,
} from "@/schemas/project.schema";
import { useMutation } from "@apollo/client";

export const useCreateProject = () => {
  const [createProjectMutation, { loading, error }] = useMutation(
    CREATE_PROJECT_MUTATION,
    {
      update(cache, { data }) {
        // add the project to the me query (for the navbar)
        const meQuery = cache.readQuery({ query: ME_QUERY }) as {
          me: {
            projects: Project[];
          };
        };
        const projects: Project[] = meQuery.me.projects;

        console.log(data.createProject);

        const newProject = ProjectSchema.safeParse(
          data.createProject
        );

        if (newProject.error) {
          throw newProject.error;
        }

        const updatedProjects = [newProject.data, ...projects];
        console.log(updatedProjects);
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
