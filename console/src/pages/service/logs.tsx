import { LogsChart } from "@/components/logs/logs-chart.comp";
import { LogsRender } from "@/components/logs/logs.comp";
import { LOG_BY_PROJECT_ID } from "@/gql/log.graphql";
import { liveLogs } from "@/state/live-log.state";
import { cn } from "@/utils";
import { useQuery } from "@apollo/client";
import { LightningBoltIcon, MagnifyingGlassIcon } from "@radix-ui/react-icons";
import { ClockIcon, RefreshCcw } from "lucide-react";
import { useEffect, useRef } from "react";
import { useParams } from "react-router-dom";
import { useSnapshot } from "valtio";

const LogsHeader = () => (
  <div
    className="border-b py-1 pl-2 font-normal text-gray-400 text-sm"
    style={{
      display: "grid",
      gridTemplateColumns:
        "auto minmax(0px, 120px) minmax(0px, 120px) minmax(0px, 200px) minmax(0px, 1fr)",
      gridAutoRows: "auto",
    }}
  >
    <div>
      <div className="scrollbar-hide h-full w-full min-w-[0px] cursor-default select-none truncate pr-2 text-xs">
        <a className="block h-full w-2 rounded-[3px]" />
      </div>
    </div>
    <p>Date</p>
    <p>Service ID</p>
    <p>Deployment Name</p>
    <p>Message</p>
  </div>
);

export const LogsPage = () => {
  const { projectId } = useParams();

  if (!projectId) {
    throw new Error("No project ID found");
  }

  const { data, subscribeToMore } = useQuery(LOG_BY_PROJECT_ID, {
    variables: {
      projectId,
      // todo: change that
      since: "2021-01-01",
      limit: 100,
    },
    pollInterval: 1000,
  });

  const { isLive, toggleLive, updateSearchQuery, searchQuery } =
    useSnapshot(liveLogs);
  const searchQueryRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    document.addEventListener("keydown", (event) => {
      if (event.ctrlKey && event.key.toLowerCase() === "f") {
        event.preventDefault();
        searchQueryRef.current?.focus();
      }
    });
  }, []);

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-row items-center gap-x-2 px-2 py-2 text-gray-700 text-sm">
        <MagnifyingGlassIcon className="ml-1 h-4 w-4" />
        <input
          ref={searchQueryRef}
          className="bg-transparent outline-non placeholder:text-gray-500 focus:outline-none"
          placeholder="Search logs..."
          onChange={(e) => updateSearchQuery(e.target.value)}
          value={searchQuery}
        />
        <div className="grow" />
        <div className="flex max-h-[28px] cursor-pointer select-none flex-row items-center gap-x-1 rounded-md border border-gray-300 bg-white p-1 px-2 text-gray-600 shadow-sm">
          <ClockIcon size={14} />
          Time range
        </div>
        <div
          title="Clear logs"
          className="flex h-[28px] w-[28px] cursor-pointer items-center justify-center gap-x-1 rounded-md border border-gray-300 bg-white p-1 shadow-sm"
        >
          <RefreshCcw size={13} />
        </div>
        <div
          title="Live logs"
          className={cn(
            isLive
              ? "border-gray-100 bg-yellow-400 text-white shadow-md shadow-yellow-200"
              : "border-gray-300 bg-white",
            "flex h-[28px] w-[28px] cursor-pointer items-center justify-center gap-x-1 rounded-md border p-1 shadow-sm transition-all"
          )}
          onClick={() => toggleLive()}
        >
          <LightningBoltIcon />
        </div>
      </div>
      <LogsChart />
      <LogsHeader />
      <LogsRender
        logs={data?.projectLogs ?? []}
        logsSubscription={() =>
          subscribeToMore({
            document: LOG_BY_PROJECT_ID,
            variables: {
              projectId,
              since: "2021-01-01",
              limit: 100,
            },
            updateQuery: (prev, { subscriptionData }) => {
              if (!subscriptionData.data) {
                return prev;
              }
              const newLogs = subscriptionData.data.projectLogs;
              const data = prev.projectLogs;

              return {
                projectLogs: [...data, ...newLogs],
              };
            },
          })
        }
      />
    </div>
  );
};
