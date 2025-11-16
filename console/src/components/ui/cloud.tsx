import { DiGoogleCloudPlatform } from "react-icons/di";
import { FaAws } from "react-icons/fa";
import { VscAzure } from "react-icons/vsc";
import { CloudProvider } from "@/_apollo/graphql";
import { cn } from "@/utils";

type CloudIconProps = React.HTMLAttributes<HTMLDivElement> & {
  cloudProvider: CloudProvider;
};

export const CloudIcon = ({ cloudProvider, className }: CloudIconProps) => {
  switch (cloudProvider) {
    case CloudProvider.Aws:
      return <FaAws className={cn("h-4 w-4", className)} />;
    case CloudProvider.Azure:
      return <VscAzure className={cn("h-4 w-4", className)} />;
    case CloudProvider.Gcp:
      return <DiGoogleCloudPlatform className={cn("h-4 w-4", className)} />;
    default:
      throw new Error("Icon not supported");
  }
};
