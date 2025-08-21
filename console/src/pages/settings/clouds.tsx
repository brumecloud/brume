import { Button } from "@/components/ui/button";
import { cn } from "@/utils";
import { AiOutlineCloudServer } from "react-icons/ai";
import { DiGoogleCloudPlatform } from "react-icons/di";
import { FaAws } from "react-icons/fa";
import type { IconType } from "react-icons/lib";
import { VscAzure } from "react-icons/vsc";
import { Link } from "react-router-dom";

const CloudCard = ({
  name,
  icon,
  disabled,
}: {
  name: string;
  icon: IconType;
  disabled?: boolean;
}) => {
  const Icon = icon;
  return (
    <div
      className={cn(
        "flex w-full flex-row items-center justify-between rounded border px-3 py-2",
        disabled && "cursor-not-allowed border-gray-200"
      )}>
      <div className="flex flex-row items-center justify-center gap-4">
        <div className="rounded-full border bg-gray-200 p-2">
          <Icon
            className={cn("size-5", disabled && "text-gray-400")}
          />
        </div>
        <div className="flex flex-col">
          <h2
            className={cn(
              "mb-0 text-sm font-semibold",
              disabled && "text-gray-400"
            )}>
            {name}
          </h2>
          <p
            className={cn(
              "mt-0 pt-0 text-xs text-gray-500",
              disabled && "text-gray-400"
            )}>
            Connect any of your {name} account {disabled && "soon"}
          </p>
        </div>
      </div>
      <Link to={`/settings/cloud/${name.toLowerCase()}`}>
        <Button size="sm" variant="outline" disabled={disabled}>
          Connect
        </Button>
      </Link>
    </div>
  );
};

export const CloudsPage = () => {
  return (
    <div className="flex h-full min-h-[calc(65vh)] px-32 pt-8">
      <div className="flex min-h-32 w-full flex-col gap-8 rounded-sm border p-2 pb-16">
        {/* header */}
        <div className="flex flex-row items-center justify-center gap-8 pt-16">
          <div className="flex flex-col items-center gap-2 pt-8">
            <div className="flex size-12 flex-col items-center justify-center rounded border">
              <AiOutlineCloudServer className="size-8" />
            </div>
            <h2 className="text-lg font-medium">
              Connect your cloud
            </h2>
            <p className="text-sm text-gray-500">
              Connect one of your cloud account through a highly
              secure connection
            </p>
          </div>
        </div>
        <div className="max-w-1/2 m-auto flex w-8/12 flex-col gap-2">
          <CloudCard name="AWS" icon={FaAws} />
          <CloudCard
            name="GCP"
            icon={DiGoogleCloudPlatform}
            disabled
          />
          <CloudCard name="Azure" icon={VscAzure} disabled />
        </div>
      </div>
    </div>
  );
};
