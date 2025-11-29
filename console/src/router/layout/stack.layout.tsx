import { useQuery } from "@apollo/client";
import { Link, Outlet, useLocation, useParams } from "react-router-dom";
import { StackStatus } from "@/_apollo/graphql";
import { Page } from "@/components/page-comp/header";
import { Badge } from "@/components/ui/badge";
import { StatusIndicator } from "@/components/ui/status";
import { GET_STACK } from "@/gql/stack.graphql";
import { cn } from "@/utils";

export const StackLayout = () => {
  const location = useLocation();
  const path = location.pathname.split("/").pop();
  const { stackId } = useParams();

  const { data, loading } = useQuery(GET_STACK, {
    variables: {
      id: stackId || "",
    },
  });

  if (loading || !data) {
    return null;
  }

  const stack = data?.getStack;

  const status = {
    [StackStatus.Deployed]: "online",
    [StackStatus.Deploying]: "pending",
    [StackStatus.Failed]: "offline",
    [StackStatus.Pending]: "pending",
    "": "offline",
  };

  return (
    <Page.Container>
      <Page.Header className="w-full">
        <div className="flex flex-row items-center gap-2">
          <Page.Title className="w-full">
            <div className="flex flex-row items-center gap-4">
              Stack - {stack.name}
            </div>
          </Page.Title>
          <Badge
            className="flex flex-row items-center gap-2"
            variant="secondary"
          >
            <StatusIndicator className={cn("group", status[stack.status])} />
            {status[stack.status] || "Offline"}
          </Badge>
        </div>
        <Page.Description>
          Explore the details of this stack. See what is running, on what
          services, logs etc.
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
              data-state={path === "logs" ? "active" : ""}
              to="logs"
            >
              Logs
            </Link>
            <Link
              className="select-none border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800"
              data-state={path === "template" ? "active" : ""}
              to="template"
            >
              Template
            </Link>
          </div>
          <Outlet />
        </div>
      </Page.Body>
    </Page.Container>
  );
};
