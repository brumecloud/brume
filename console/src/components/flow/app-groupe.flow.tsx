import { Boxes, EllipsisVertical } from "lucide-react";
import type { CSSProperties } from "react";

export default function AppGroupFlow({
  data,
  style,
  ...props
}: {
  data: { label: string };
  style: CSSProperties;
  width: number;
  height: number;
}) {
  return (
    <div
      style={{
        ...style,
        width: props.width,
        height: props.height,
      }}
      {...props}
      className="flex flex-col rounded-xl border border-slate-200 backdrop-blur-[2px]">
      <div className="flex flex-row items-center justify-between px-5 py-4 pr-3">
        <div className="flex flex-row items-center gap-2">
          <Boxes
            strokeWidth={1.5}
            className="h-5 w-5 text-gray-800"
          />
          <div className="text-sm font-normal">{data.label}</div>
        </div>
        <div>
          <EllipsisVertical
            strokeWidth={1.5}
            className="h-5 w-5 text-gray-800"
          />
        </div>
      </div>
    </div>
  );
}
