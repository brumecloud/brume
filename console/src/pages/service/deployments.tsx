import { useFragment } from "@apollo/client";
import { GitBranch, GitCommit } from "lucide-react";
import { useParams } from "react-router-dom";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { DEPLOYMENT_FRAGMENT } from "@/gql/deployment.graphql";
import { ServiceFragment } from "@/gql/service.graphql";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

const DeploymentCard = ({
  id,
  isLive,
  className,
}: {
  id: string;
  isLive?: boolean;
  className?: string;
}) => {
  const { data, complete } = useFragment({
    fragment: DEPLOYMENT_FRAGMENT,
    from: `Deployment:${id}`,
    fragmentName: "DeploymentFragment",
  });

  if (!data) {
    return null;
  }

  if (!complete) {
    throw new Error("Deployment not complete");
  }

  const status = data.logs.status;
  const readyState =
    status === "success" ? "ready" : status === "failed" ? "error" : "loading";
  const deploymentTime = `${data.logs.duration}s`;
  // age of the deployment

  return (
    <div className={cn("gap-x-2 px-6 py-4", className)}>
      <div className="grid grid-cols-10 items-center justify-between">
        <div className="col-span-3">
          <div>
            <p className="font-semibold text-gray-700 text-sm">
              {data.id.slice(0, 8)}
            </p>
            <div className="flex flex-row items-center gap-x-3">
              <p className="text-gray-400 text-sm">{data.env}</p>
              {isLive && (
                <Badge
                  variant="outline"
                  className={"border-blue-400/50 bg-blue-300 text-white"}
                >
                  Live
                </Badge>
              )}
            </div>
          </div>
        </div>
        <div className="col-span-2 flex max-w-[500px] flex-row items-center gap-x-4">
          <div className="grid gap-y-2">
            <div>
              <Badge
                variant="secondary"
                className={`${
                  readyState === "ready"
                    ? "bg-green-300"
                    : readyState === "error"
                      ? "bg-red-300"
                      : "bg-yellow-300"
                } text-white`}
              >
                {readyState}
              </Badge>
            </div>
            <p className="text-gray-500 text-sm">{`took ${deploymentTime}`}</p>
          </div>
        </div>
        {data.source.type === "git" && (
          <div className="col-span-3 grid gap-y-2">
            <div className="grid grid-cols-[auto_1fr] items-center gap-x-2 font-mono text-sm">
              <GitBranch className="h-4 w-4" />
              {data.source.branch}
            </div>
            <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 text-sm">
              <div className="grid grid-cols-[auto_1fr] items-center gap-x-2">
                <GitCommit className="h-4 w-4" />
                <span className="font-mono">{data.source.commit}</span>
              </div>
              <p className="max-w-[300px] truncate text-gray-500 text-sm">
                {data.source.message}
              </p>
            </div>
          </div>
        )}
        {data.source.type === "console" && (
          <div className="col-span-3 grid gap-y-2">
            <p className="text-gray-500 text-sm">Manual Deployment</p>
            <p className="text-gray-400 text-sm italic">
              Deploy manually inside Brume console
            </p>
          </div>
        )}
        <div className="col-end-11 justify-items-end">
          <Avatar className="h-8 w-8">
            <AvatarImage src={data.author.avatar} />
            <AvatarFallback>{data.author.name.slice(0, 2)}</AvatarFallback>
          </Avatar>
        </div>
      </div>
    </div>
  );
};

// we are in a subpage of the project route
// we should be working on fragments
export const DeploymentsPage = () => {
  const { serviceId } = useParams<RouteParams>();

  const { data, complete } = useFragment({
    from: `Service:${serviceId}`,
    fragment: ServiceFragment,
    fragmentName: "ServiceFragment",
  });

  if (!data) {
    return null;
  }

  if (!complete) {
    throw new Error("Service not complete");
  }

  const deploymentsRaw = data.deployments;

  const deployments = deploymentsRaw.sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
  );

  return (
    <div className="flex h-full flex-col gap-y-4 px-32 pt-8">
      <div className="flex h-10 flex-row items-center">
        <h3 className="font-medium text-md">Live Deployment</h3>
      </div>
      {deployments[0]?.id && (
        <div className="rounded-sm border border-gray-200 bg-white">
          <DeploymentCard id={deployments[0].id} isLive />
        </div>
      )}
      {deployments.length > 1 && (
        <>
          <h3 className="pt-4 font-medium text-md">History</h3>
          <div className="flex h-full flex-col rounded-md border border-gray-200 bg-white shadow-sm">
            {deployments.slice(1).map((deployment) => (
              <DeploymentCard
                className="border-gray-200 border-b last:border-b-0"
                id={deployment.id}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
};
