import { cn } from "@/utils";
import { Collapsable } from "@shadcn/collapsable";
import { Play } from "lucide-react";
import { useState, type PropsWithChildren } from "react";

type Props = {
  title: string;
  initialIsOpen?: boolean;
  className?: string;
  indent?: number;
  prefix?: React.ReactNode;
} & PropsWithChildren;

export const CollapsibleWrapper = ({ children, title, initialIsOpen, indent = 2, prefix = null }: Props) => {
  const [isOpen, setIsOpen] = useState<boolean>(initialIsOpen ?? false);

  const onSectionTitleClick = () => {
    setIsOpen((prev) => !prev);
  };

  return (
    <section className="box-border w-full text-[13px]">
      <div className="flex w-[calc(100%-0.75rem)] items-center justify-between gap-2 pb-1 font-medium">
        {prefix}
        <div
          className="flex cursor-pointer select-none items-center gap-2 rounded hover:bg-gray-100"
          onClick={onSectionTitleClick}>
          <span className="text-gray-800">{title}</span>
          <Play
            fill="current"
            size={8}
            className={cn(`fill-gray-800 text-gray-800 transition-transform`, {
              "rotate-90": isOpen,
            })}
          />
        </div>
      </div>
      <Collapsable
        isOpen={isOpen}
        className={cn("box-border w-[calc(100%-1.5rem)]", indent && `pl-${indent}`)}>
        {children}
      </Collapsable>
    </section>
  );
};
