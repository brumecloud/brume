import { cn } from "@/utils";

type StatusProps = React.HTMLAttributes<HTMLDivElement> & {
  status: "ok" | "cancelled" | "error" | "pending" | "warning";
};

export function Status({ status, ...props }: StatusProps) {
  const color = {
    ok: "bg-green-500",
    cancelled: "bg-red-500",
    error: "bg-red-500",
    pending: "bg-yellow-500",
    warning: "bg-yellow-500",
  };

  return (
    <div className={cn("rounded-full border p-1", color[status])} {...props} />
  );
}
