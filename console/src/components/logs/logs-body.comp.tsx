import type { Log } from "@/_apollo/graphql";

export const LogLine = ({ data }: { data: Log }) => {
  function formatDate(date: Date): string {
    const months = [
      "Jan",
      "Feb",
      "Mar",
      "Apr",
      "May",
      "Jun",
      "Jul",
      "Aug",
      "Sep",
      "Oct",
      "Nov",
      "Dec",
    ];

    const month = months[date.getMonth()];
    const day = date.getDate().toString().padStart(2, "0");
    const hours = date.getHours().toString().padStart(2, "0");
    const minutes = date.getMinutes().toString().padStart(2, "0");
    const seconds = date.getSeconds().toString().padStart(2, "0");

    return `${month} ${day} ${hours}:${minutes}:${seconds}`;
  }

  return (
    <div className="group/logline group/logline relative w-full rounded border border-transparent py-[0.2em]">
      <div className="relative z-10 w-full pl-2">
        <div
          style={{
            display: "grid",
            gridTemplateColumns:
              "auto minmax(0px, 120px) minmax(0px, 120px) minmax(0px, 200px) minmax(0px, 1fr)",
            gridAutoRows: "auto",
          }}>
          <div>
            <div
              className="scrollbar-hide h-full w-full min-w-[0px] cursor-default select-none truncate pr-2 text-xs"
              title="Info">
              <a className="block h-full w-2 rounded-[3px] bg-blue-200 hover:bg-blue-400 focus:outline-none focus-visible:ring-2 focus-visible:ring-pink-700"></a>
            </div>
          </div>
          <div>
            <div className="scrollbar-hide min-w-[0px] cursor-default select-none truncate text-xs text-gray-500">
              <a
                title="Jul 28 2024, 11:33 AM (GMT+2)"
                className="hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-pink-700">
                {formatDate(new Date(data.timestamp))}
              </a>
            </div>
          </div>
          <div>
            <div className="scrollbar-hide min-w-[0px] cursor-default select-none truncate pr-2 text-xs text-gray-600">
              <a
                className="cursor:pointer hover:text-gray-600 hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-pink-700"
                href="#">
                {data.serviceId}
              </a>
            </div>
          </div>
          <div>
            <div className="scrollbar-hide min-w-[0px] cursor-default select-none truncate pr-2 text-xs text-gray-600">
              <a
                className="cursor:pointer hover:text-gray-600 hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-pink-700"
                href="#">
                {data.deploymentName}
              </a>
            </div>
          </div>
          <div>
            <div className="scrollbar-hide min-w-[0px] truncate whitespace-pre-wrap text-xs">
              {data.message}
            </div>
          </div>
        </div>
      </div>
      <div
        aria-hidden="true"
        className="absolute inset-0 opacity-30 group-hover/logline:bg-gray-100 group-hover/logline:opacity-40"></div>
    </div>
  );
};
