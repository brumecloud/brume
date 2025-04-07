import { gql } from "@/_apollo/gql";
import {
  ProjectSchema,
  type Project,
} from "@/schemas/project.schema";
import { useQuery } from "@apollo/client";
import { z } from "zod";

const PROJECTS_QUERY = gql(`
  query myProjects {
    me {
      projects {
        ...ProjectFragment
      }
    }
  }
`);

const ProjectListSchema = z.array(ProjectSchema);

export const useProjects = (): {
  projects: Project[];
  loading: boolean;
  error?: Error | null;
} => {
  const { data, loading } = useQuery(PROJECTS_QUERY, {
    // data will come from the me query
    fetchPolicy: "cache-only",
  });

  if (loading || !data) {
    return {
      projects: [],
      loading: true,
    };
  } else {
    const rawData = ProjectListSchema.safeParse(data?.me?.projects);

    if (!rawData.success) {
      throw new Error(rawData.error.message);
    } else {
      return {
        projects: rawData.data,
        loading: false,
      };
    }
  }
};
