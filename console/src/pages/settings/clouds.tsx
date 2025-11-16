import { useFragment, useQuery } from "@apollo/client";
import { CloudIcon } from "lucide-react";
import { AiOutlineCloudServer } from "react-icons/ai";
import { DiGoogleCloudPlatform } from "react-icons/di";
import { FaAws } from "react-icons/fa";
import type { IconType } from "react-icons/lib";
import { VscAzure } from "react-icons/vsc";
import { Link, useNavigate } from "react-router-dom";
import { CloudProvider, CloudStatus } from "@/_apollo/graphql";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "@/components/ui/select";
import { Status } from "@/components/ui/status";
import {
  CLOUD_ACCOUNT_FRAGMENT,
  GET_CLOUD_ACCOUNTS,
} from "@/gql/cloud.graphql";
import { cn } from "@/utils";

const CreateCloudCard = ({
  name,
  icon,
  disabled,
}: {
  name: string;
  icon: IconType;
  disabled?: boolean;
}) => {
  const Icon = icon;
  const LinkComponent = disabled ? "div" : Link;
  return (
    <div
      className={cn(
        "flex w-full flex-row items-center justify-between rounded border px-3 py-2",
        disabled && "cursor-not-allowed border-gray-200"
      )}
    >
      <div className="flex flex-row items-center justify-center gap-4">
        <div className="rounded-full border bg-gray-200 p-2">
          <Icon className={cn("size-5", disabled && "text-gray-400")} />
        </div>
        <div className="flex flex-col">
          <h2
            className={cn(
              "mb-0 font-semibold text-sm",
              disabled && "text-gray-400"
            )}
          >
            {name}
          </h2>
          <p
            className={cn(
              "mt-0 pt-0 text-gray-500 text-xs",
              disabled && "text-gray-400"
            )}
          >
            Connect any of your {name} account {disabled && "soon"}
          </p>
        </div>
      </div>
      <LinkComponent to={`/settings/cloud/${name.toLowerCase()}`}>
        <Button disabled={disabled} size="sm" variant="outline">
          Connect
        </Button>
      </LinkComponent>
    </div>
  );
};

const _cloudProviderMapping = {
  [CloudProvider.Aws]: {
    name: "Amazon Web Services",
    icon: FaAws,
  },
};

export const _statusMapping: Record<
  CloudStatus,
  {
    status: "ok" | "error" | "warning" | "cancelled" | "pending";
    label: string;
  }
> = {
  [CloudStatus.Connected]: {
    status: "ok",
    label: "Connected",
  },
  [CloudStatus.Disconnected]: {
    status: "error",
    label: "Disconnected",
  },
  [CloudStatus.Error]: {
    status: "error",
    label: "Error",
  },
  [CloudStatus.Pending]: {
    status: "warning",
    label: "Pending",
  },
};

const CloudCard = ({ cid }: { cid: string }) => {
  const { data, complete } = useFragment({
    fragment: CLOUD_ACCOUNT_FRAGMENT,
    from: `CloudAccount:${cid}`,
  });

  if (!complete || data.cloudProvider !== CloudProvider.Aws) {
    return null;
  }

  const cloudProvider = _cloudProviderMapping[data.cloudProvider];
  const _status = _statusMapping[data.status];

  return (
    <div
      className={cn(
        "flex w-full flex-col items-center justify-between rounded-md border"
      )}
    >
      <Link
        className="flex h-full w-full flex-row items-center justify-between gap-4 bg-gray-100 px-3 py-2"
        to={`/settings/cloud/${cid}/overview`}
      >
        <div className="flex flex-row items-center gap-4">
          <div className="p-2">
            <cloudProvider.icon className={cn("size-5")} />
          </div>
          <div className="flex flex-col">
            <h2 className={cn("mb-0 font-semibold text-sm")}>{data.name}</h2>
            <p className={cn("mb-0 text-gray-500 text-xs")}>
              {cloudProvider.name}
            </p>
          </div>
        </div>
        <div>
          <Badge
            className="flex flex-row items-center gap-2"
            variant="secondary"
          >
            <Status status={_status.status} />
            {_status.label}
          </Badge>
        </div>
      </Link>
      <div className="w-full border-gray-200 border-t p-3">
        {data.stacks.length === 0 && (
          <p className="text-gray-400 text-xs">No stacks created yet</p>
        )}
        {data.stacks.map((stack) => (
          <Badge key={stack.id} variant="outline">
            {stack.name}
          </Badge>
        ))}
      </div>
    </div>
  );
};

const EmptyCloudAccount = () => (
  <div className="flex min-h-32 w-full flex-col gap-8 rounded-xl border p-2 pb-16">
    <div className="flex flex-row items-center justify-center gap-8 pt-16">
      <div className="flex flex-col items-center gap-2 pt-8">
        <div className="flex size-12 flex-col items-center justify-center rounded border">
          <AiOutlineCloudServer className="size-8" />
        </div>
        <h2 className="font-medium text-lg">Connect your cloud</h2>
        <p className="text-gray-500 text-sm">
          Connect one of your cloud account through a highly secure connection
        </p>
      </div>
    </div>
    <div className="m-auto flex w-8/12 max-w-1/2 flex-col gap-2">
      <CreateCloudCard icon={FaAws} name="AWS" />
      <CreateCloudCard disabled icon={DiGoogleCloudPlatform} name="GCP" />
      <CreateCloudCard disabled icon={VscAzure} name="Azure" />
    </div>
  </div>
);

export const CloudsPage = () => {
  const { data, loading } = useQuery(GET_CLOUD_ACCOUNTS);
  const navigate = useNavigate();

  if (loading) {
    return null;
  }

  const _cloudAccounts = data?.me?.organization?.cloudAccounts;

  return (
    <div className="flex h-full min-h-[calc(65vh)] pt-8">
      {_cloudAccounts?.length && _cloudAccounts?.length > 0 ? (
        <div className="w-full">
          <div className="m-auto flex flex-row gap-2">
            <Input
              className="w-full"
              placeholder="Search for a cloud provider"
            />
            <Select
              defaultValue=""
              onValueChange={(value) => {
                if (value) {
                  navigate(`/settings/cloud/${value}`);
                }
              }}
            >
              <SelectTrigger className="w-[155px] bg-primary text-primary-foreground">
                <CloudIcon className="h-4 w-4" />
                <span>Add Cloud</span>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="aws">
                  <div className="flex w-full items-center gap-3">
                    <FaAws className="h-4 w-4" />
                    AWS
                  </div>
                </SelectItem>
                <SelectItem disabled value="azure">
                  <div className="flex w-full items-center gap-3">
                    <VscAzure className="h-4 w-4" />
                    Azure
                  </div>
                </SelectItem>
                <SelectItem disabled value="gcp">
                  <div className="flex w-full items-center gap-3">
                    <DiGoogleCloudPlatform className="h-4 w-4" />
                    GCP
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-4 pt-8">
            {_cloudAccounts.map((cloud) => (
              <CloudCard cid={cloud.id} key={cloud.id} />
            ))}
          </div>
        </div>
      ) : (
        <EmptyCloudAccount />
      )}
    </div>
  );
};
