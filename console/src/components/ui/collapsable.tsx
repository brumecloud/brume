import { cn } from "@/utils";
import type { PropsWithChildren } from "react";

type Props = {
  isOpen: boolean;
  className?: string;
} & PropsWithChildren;

export const Collapsable = ({ children, isOpen, className }: Props) => (
  <section
    className={cn(
      "grid w-full transition-all duration-150 ease-in-out",
      {
        "grid-rows-[1fr]": isOpen,
        "grid-rows-[0fr] opacity-0": !isOpen,
      },
      className
    )}
  >
    <div className="h-full overflow-x-auto overflow-y-hidden">{children}</div>
  </section>
);
