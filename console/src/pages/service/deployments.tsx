import {
  Avatar,
  AvatarFallback,
  AvatarImage,
} from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { useUser } from "@/hooks/useUser";
import { cn } from "@/utils";
import { GitBranch, GitCommit } from "lucide-react";

interface DeploymentCardProps {
  deploymentId: string;
  environment: string;
  status?: "live";
  readyState: "ready" | "error" | "loading";
  deploymentTime: string;
  deploymentAge: string;
  branch: string;
  commitHash: string;
  commitMessage: string;
  user?: {
    avatar: string;
    name: string;
  };
  className?: string;
}

const DeploymentCard = ({
  data: {
    deploymentId,
    environment,
    status,
    readyState,
    deploymentTime,
    deploymentAge,
    branch,
    commitHash,
    commitMessage,
    user,
  },
  className,
}: {
  data: DeploymentCardProps;
  className?: string;
}) => {
  return (
    <div className={cn("gap-x-2 px-6 py-4", className)}>
      <div className="grid grid-cols-10 items-center justify-between">
        <div className="col-span-3">
          <div>
            <p className="text-sm font-semibold text-gray-700">
              {deploymentId}
            </p>
            <div className="flex flex-row items-center gap-x-3">
              <p className="text-sm text-gray-400">{environment}</p>
              {status && (
                <Badge
                  variant="outline"
                  className="bg-blue-100 text-blue-800">
                  {status}
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
            <p className="text-sm text-gray-500">{`${deploymentTime} (${deploymentAge})`}</p>
          </div>
        </div>
        <div className="col-span-3 grid gap-y-2">
          <div className="grid grid-cols-[auto_1fr] items-center gap-x-2 font-mono text-sm">
            <GitBranch className="h-4 w-4" />
            {branch}
          </div>
          <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 text-sm">
            <div className="grid grid-cols-[auto_1fr] items-center gap-x-2">
              <GitCommit className="h-4 w-4" />
              <span className="font-mono">{commitHash}</span>
            </div>
            <p className="max-w-[300px] truncate text-sm text-gray-500">
              {commitMessage}
            </p>
          </div>
        </div>
        <div className="col-end-11 justify-items-end">
          {user && (
            <Avatar className="h-8 w-8">
              <AvatarImage src={user.avatar} />
              <AvatarFallback>{user.name.slice(0, 2)}</AvatarFallback>
            </Avatar>
          )}
        </div>
      </div>
    </div>
  );
};

export const DeploymentsPage = () => {
  const { me } = useUser();

  const deployments: DeploymentCardProps[] = [
    {
      deploymentId: "jkvwuik92jr",
      environment: "Production",
      status: "live",
      readyState: "ready",
      deploymentTime: "1m 23s",
      deploymentAge: "3d ago",
      branch: "master",
      commitHash: "dfuie283j",
      commitMessage: "commit name",
      user: me ? { avatar: me.avatar, name: me.name } : undefined,
    },
    {
      deploymentId: "k39vn4m2p1",
      environment: "Staging",
      readyState: "loading",
      deploymentTime: "45s",
      deploymentAge: "1d ago",
      branch: "feature/auth",
      commitHash: "93kdf82n",
      commitMessage: "Add authentication flow",
      user: me ? { avatar: me.avatar, name: me.name } : undefined,
    },
    {
      deploymentId: "p2m4n8v1k3",
      environment: "Development",
      readyState: "error",
      deploymentTime: "2m 15s",
      deploymentAge: "5h ago",
      branch: "fix/api-endpoint",
      commitHash: "7jh2k4f9",
      commitMessage: "Fix API endpoint validation",
      user: me ? { avatar: me.avatar, name: me.name } : undefined,
    },
    {
      deploymentId: "w5x8y2z4q7",
      environment: "Preview",
      readyState: "ready",
      deploymentTime: "3m 40s",
      deploymentAge: "12h ago",
      branch: "feature/dashboard",
      commitHash: "n4m6p8q2",
      commitMessage: "Update dashboard components",
      user: me ? { avatar: me.avatar, name: me.name } : undefined,
    },
  ];

  return (
    <div className="flex h-full flex-col gap-y-4 px-32 pt-8">
      <div className="flex h-10 flex-row items-center">
        <h3 className="text-md font-medium">Live Deployment</h3>
      </div>
      <div className="rounded-sm border border-gray-200 bg-white">
        <DeploymentCard
          data={deployments[0] as DeploymentCardProps}
        />
      </div>
      <h3 className="text-md pt-4 font-medium">History</h3>
      <div className="flex h-full flex-col rounded-md border border-gray-200 bg-white shadow-sm">
        {deployments.slice(1).map((deployment) => (
          <DeploymentCard
            className="border-b border-gray-200 last:border-b-0"
            data={deployment}
          />
        ))}
      </div>
    </div>
  );
};
