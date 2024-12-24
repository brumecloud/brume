import type { CSSProperties } from "react";

export default function MachineNodeFlow({
  style,
  ...props
}: {
  style: CSSProperties;
  width: number;
  height: number;
}) {
  return (
    <div
      style={{ ...style, width: props.width, height: props.height }}
      className="rounded-sm border border-gray-200 bg-gray-100"
      {...props}>
      <div className="flex w-full items-center justify-between p-2">
        <div className="flex flex-col">
          <p className="text-xs font-normal text-gray-500">
            Machine:
          </p>
          <p className="text-xs font-normal text-gray-800">
            docker-local-machine
          </p>
        </div>
      </div>
    </div>
  );
}
