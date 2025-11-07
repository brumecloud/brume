import { cn } from "@/utils";

const PageHeader = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
    <div
      className={cn(
        "flex flex-row items-center justify-between pt-16",
        className
      )}
      {...props}
    >
      <div className="flex flex-col pb-8">{children}</div>
    </div>
  );

const PageHeaderTitle = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLHeadingElement>) => (
    <h2 className={cn("pb-2 font-heading text-3xl", className)} {...props}>
      {children}
    </h2>
  );

const PageHeaderDescription = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLParagraphElement>) => (
    <p className={cn("text-gray-500 text-sm", className)} {...props}>
      {children}
    </p>
  );

const PageContainer = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
    <div className={cn("flex h-full flex-col", className)} {...props}>
      <div className="h-full px-32 pt-8">{children}</div>
    </div>
  );

const PageBody = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
    <div className={cn("flex h-full flex-col", className)} {...props}>
      {children}
    </div>
  );

export const Page = {
  Container: PageContainer,
  Header: PageHeader,
  Title: PageHeaderTitle,
  Description: PageHeaderDescription,
  Body: PageBody,
};
