import { FaGithub, FaGitlab, FaGitSquare } from "react-icons/fa";
import type { IconType } from "react-icons/lib";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { cn } from "@/utils";

const GitProviderCard = ({
  name,
  icon,
  disabled,
  className,
}: {
  name: string;
  icon: IconType;
  disabled?: boolean;
  className?: string;
}) => {
  const Icon = icon;
  return (
    <div
      className={cn(
        "flex w-full flex-row items-center justify-between rounded border px-3 py-2",
        disabled && "cursor-not-allowed border-gray-200",
        className
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
      <Link to={`/settings/cloud/${name.toLowerCase()}`}>
        <Button disabled={disabled} size="sm" variant="outline">
          Connect
        </Button>
      </Link>
    </div>
  );
};

export const GitPage = () => {
  return (
    <div className="flex h-full min-h-[calc(65vh)] pt-8">
      <div className="flex min-h-32 w-full flex-col gap-8 rounded-xl border p-2 pb-16">
        {/* header */}
        <div className="flex flex-row items-center justify-center gap-8 pt-16">
          <div className="flex flex-col items-center gap-2 pt-8">
            <div className="flex size-12 flex-col items-center justify-center rounded border">
              <FaGitSquare className="size-8" />
            </div>
            <h2 className="font-medium text-lg">Connect your git provider</h2>
            <p className="text-gray-500 text-sm">
              Connect the place where you push your code to. (you can also
              trigger builds from cli if needed)
            </p>
          </div>
        </div>
        <div className="m-auto flex w-8/12 max-w-1/2 flex-col gap-2">
          <GitProviderCard icon={FaGithub} name="Github" />
          <GitProviderCard disabled icon={FaGitlab} name="Gitlab" />
        </div>
      </div>
    </div>
  );
};
