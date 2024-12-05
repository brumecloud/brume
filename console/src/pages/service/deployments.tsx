import {
  Avatar,
  AvatarFallback,
  AvatarImage,
} from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { useService } from "@/hooks/useService";
import { useUser } from "@/hooks/useUser";
import type { Deployment } from "@/schemas/service.schema";
import { cn } from "@/utils";
import { GitBranch, GitCommit } from "lucide-react";

const DeploymentCard = ({
  data,
  isLive,
  className,
}: {
  data: Deployment;
  isLive?: boolean;
  className?: string;
}) => {
  const { me } = useUser();

  const status = data.logs.status;
  const readyState =
    status === "success"
      ? "ready"
      : status === "failed"
        ? "error"
        : "loading";
  const deploymentTime = `${data.logs.duration}s`;
  // age of the deployment

  return (
    <div className={cn("gap-x-2 px-6 py-4", className)}>
      <div className="grid grid-cols-10 items-center justify-between">
        <div className="col-span-3">
          <div>
            <p className="text-sm font-semibold text-gray-700">
              {data.id.slice(0, 8)}
            </p>
            <div className="flex flex-row items-center gap-x-3">
              <p className="text-sm text-gray-400">{data.env}</p>
              {isLive && (
                <Badge
                  variant="outline"
                  className={`border-blue-400/50 bg-blue-300 text-white`}>
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
                } text-white`}>
                {readyState}
              </Badge>
            </div>
            <p className="text-sm text-gray-500">{`took ${deploymentTime}`}</p>
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
                <span className="font-mono">
                  {data.source.commit}
                </span>
              </div>
              <p className="max-w-[300px] truncate text-sm text-gray-500">
                {data.source.message}
              </p>
            </div>
          </div>
        )}
        {data.source.type === "console" && (
          <div className="col-span-3 grid gap-y-2">
            <p className="text-sm text-gray-500">Manual Deployment</p>
            <p className="text-sm italic text-gray-400">
              Deploy manually inside Brume console
            </p>
          </div>
        )}
        <div className="col-end-11 justify-items-end">
          {me && (
            <Avatar className="h-8 w-8">
              <AvatarImage src={me.avatar} />
              <AvatarFallback>{me.name.slice(0, 2)}</AvatarFallback>
            </Avatar>
          )}
        </div>
      </div>
    </div>
  );
};

export const DeploymentsPage = () => {
  const { service } = useService();

  if (!service || service.deployments.length === 0) {
    return null;
  }

  const deployments = service.deployments.sort(
    (a, b) =>
      new Date(b.createdAt).getTime() -
      new Date(a.createdAt).getTime()
  );

  return (
    <div className="flex h-full flex-col gap-y-4 px-32 pt-8">
      <div className="flex h-10 flex-row items-center">
        <h3 className="text-md font-medium">Live Deployment</h3>
      </div>
      <div className="rounded-sm border border-gray-200 bg-white">
        <DeploymentCard data={deployments[0]} isLive />
      </div>
      {deployments.length > 1 && (
        <>
          <h3 className="text-md pt-4 font-medium">History</h3>
          <div className="flex h-full flex-col rounded-md border border-gray-200 bg-white shadow-sm">
            {service.deployments.slice(1).map((deployment) => (
              <DeploymentCard
                className="border-b border-gray-200 last:border-b-0"
                data={deployment}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
};
