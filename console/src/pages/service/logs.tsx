import { LogsChart } from "@/components/logs/logs-chart.comp";
import { LogsRender } from "@/components/logs/logs.comp";

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
  return (
    <div className="flex h-full flex-col">
      <LogsChart />
      <LogsHeader />
      <LogsRender />
    </div>
  );
};
