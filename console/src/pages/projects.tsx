import { ProjectCard } from "@/components/cards/projects";
import { useUser } from "@/hooks/useUser";
import { ArrowRight } from "lucide-react";

export function Projects() {
  const { me } = useUser();

  return (
    <div className="mx-16">
      <div>
        <h1 className="select-none py-16 text-xl">
          Hello{" "}
          <span className="font-bold italic transition-all ease-in">
            {me?.name}
          </span>
        </h1>
      </div>
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
