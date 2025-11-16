import { useQuery } from "@apollo/client";
import { Link, Outlet, useLocation, useParams } from "react-router-dom";
import { Page } from "@/components/page-comp/header";
import { Badge } from "@/components/ui/badge";
import { CloudIcon } from "@/components/ui/cloud";
import { Status } from "@/components/ui/status";
import { GET_CLOUD_ACCOUNT } from "@/gql/cloud.graphql";
import { _statusMapping } from "@/pages/settings/clouds";

export const CloudLayout = () => {
  const location = useLocation();
  const path = location.pathname.split("/").pop();
  const { cloudAccountId } = useParams() as {
    cloudAccountId: string;
  };

  const { data, loading } = useQuery(GET_CLOUD_ACCOUNT, {
    variables: {
      id: cloudAccountId,
    },
    fetchPolicy: "cache-and-network",
  });

  if (loading || !data) {
    return null;
  }

  const _status = _statusMapping[data.getCloudAccountById.status];

  return (
    <Page.Container>
      <Page.Header className="w-full">
        <div className="flex flex-row items-center gap-2">
          <Page.Title className="w-full">
            <div className="flex flex-row items-center gap-4">
              <CloudIcon
                className="size-8"
                cloudProvider={data?.getCloudAccountById?.cloudProvider}
              />
              {data?.getCloudAccountById.name}
            </div>
          </Page.Title>
          <Badge
            className="flex flex-row items-center gap-2"
            variant="secondary"
          >
            <Status status={_status.status} />
            {_status.label}
          </Badge>
        </div>
        <Page.Description>
          Explore the details of this cloud account. See what is running, on
          what stacks.
        </Page.Description>
      </Page.Header>
      <Page.Body>
        <div>
          <div className="flex shrink-0 gap-6 border-b">
            <Link
              className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
              data-state={path === "overview" ? "active" : ""}
              to="overview"
            >
              Overview
            </Link>
            <Link
              className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
              data-state={path === "stacks" ? "active" : ""}
              to="stacks"
            >
              Stacks
            </Link>
            <Link
              className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
              data-state={path === "settings" ? "active" : ""}
              to="settings"
            >
              Settings
            </Link>
          </div>
          <Outlet />
        </div>
      </Page.Body>
    </Page.Container>
  );
};
