import { LogsChart } from "@/components/logs/logs-chart.comp";
import { LogsRender } from "@/components/logs/logs.comp";
import { gql, useSubscription } from "@apollo/client";
import { LightningBoltIcon, MagnifyingGlassIcon } from "@radix-ui/react-icons";
import { ClockIcon, RefreshCcw } from "lucide-react";

const LogsHeader = () => {
  return (
    <div
      className="border-b py-1 pl-2 text-sm font-normal text-gray-400"
      style={{
        display: "grid",
        gridTemplateColumns: "auto minmax(0px, 120px) minmax(0px, 120px) minmax(0px, 200px) minmax(0px, 1fr)",
        gridAutoRows: "auto",
      }}>
      <div>
        <div className="scrollbar-hide h-full w-full min-w-[0px] cursor-default select-none truncate pr-2 text-xs">
          <a className="block h-full w-2 rounded-[3px]"></a>
        </div>
      </div>
      <p>Date</p>
      <p>User</p>
      <p>Repo</p>
      <p>Message</p>
    </div>
  );
};

export const LogsPage = () => {
  const { data, loading } = useSubscription(
    gql`
      subscription Logs($serviceId: String!) {
        serviceLogs(serviceId: $serviceId) {
          level
          message
        }
      }
    `,
    {
      onSubscriptionComplete: () => {
        console.log("Subscription finished");
      },
      variables: {
        serviceId: "",
      },
    }
  );

  console.log(data, loading);

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-row items-center gap-x-2 px-2 py-2 text-sm text-gray-700">
        <MagnifyingGlassIcon className="ml-1 h-4 w-4" />
        <input
          className="outline-non bg-transparent placeholder:text-gray-500 focus:outline-none"
          placeholder="Search logs..."
        />
        <div className="grow" />
        <div className="flex max-h-[28px] cursor-pointer select-none flex-row items-center gap-x-1 rounded-md border border-gray-300 bg-white p-1 px-2 text-gray-600 shadow-sm">
          <ClockIcon size={14} />
          Time range
        </div>
        <div className="flex h-[28px] w-[28px] cursor-pointer items-center justify-center gap-x-1 rounded-md border border-gray-300 bg-white p-1 shadow-sm">
          <RefreshCcw size={13} />
        </div>
        <div className="flex h-[28px] w-[28px] cursor-pointer items-center justify-center gap-x-1 rounded-md border border-gray-300 bg-white p-1 shadow-sm">
          <LightningBoltIcon />
        </div>
      </div>
      <LogsChart />
      <LogsHeader />
      <LogsRender />
    </div>
  );
};
