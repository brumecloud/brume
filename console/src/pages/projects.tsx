import type { UserFragmentFragment } from "@/_apollo/graphql";
import { ProjectCard } from "@/components/cards/projects";
import { ME_QUERY, USER_FRAGMENT } from "@/gql/user.graphql";
import {
  useFragment,
  useQuery,
  type FragmentType,
} from "@apollo/client";
import { ArrowRight } from "lucide-react";

type ProjectHeaderProps = {
  projectData: FragmentType<UserFragmentFragment>;
};

function ProjectHeader({ projectData }: ProjectHeaderProps) {
  const { data, complete } = useFragment({
    fragment: USER_FRAGMENT,
    from: projectData,
  });

  if (!complete) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h1 className="select-none py-16 text-xl">
        Hello{" "}
        <span className="font-bold italic transition-all ease-in">
          {data.name}
        </span>
      </h1>
    </div>
  );
}

export function Projects() {
  const { data, loading } = useQuery(ME_QUERY, {
    fetchPolicy: "cache-first",
  });

  if (loading) {
    return <div>Loading...</div>;
  }

  if (!data) {
    throw new Error("No data for the current user ?");
  }

  return (
    <div className="mx-16">
      <ProjectHeader projectData={data.me} />
      <div className="flex flex-col gap-16">
        <div>
          <h2 className="flex select-none flex-row items-center gap-x-2 py-4">
            Stratumn
            <ArrowRight size={20} />
          </h2>
          <div className="grid grid-cols-3 gap-x-4 gap-y-4">
            <ProjectCard name="Trace-API" url="trace-api.stratumn" />
            <ProjectCard
              name="Account-API"
              url="account-api.stratumn"
            />
            <ProjectCard
              name="Studio-API"
              url="studio-api.stratumn"
            />
            <ProjectCard name="Trace UI" url="trace.stratumn.com" />
            <ProjectCard name="Studio UI" url="studio.stratumn.com" />
          </div>
        </div>
        <div>
          <h2 className="flex select-none flex-row items-center gap-x-2 py-4">
            SiaGPT
            <ArrowRight size={20} />
          </h2>
          <div className="grid grid-cols-3 gap-x-4 gap-y-4">
            <ProjectCard name="Backend" url="user-api.brume" />
            <ProjectCard name="AI" url="ai.brume" />
            <ProjectCard name="Frontend" url="https://planchon.io" />
          </div>
        </div>
      </div>
    </div>
  );
}
