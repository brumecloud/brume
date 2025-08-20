import { cn } from "@/utils";

const PageHeader = ({
	children,
	className,
	...props
}: React.HTMLAttributes<HTMLDivElement>) => {
	return (
		<div
			className={cn(
				"flex flex-row items-center justify-between pt-16",
				className,
			)}
			{...props}
		>
			<div className="flex flex-col pb-8">{children}</div>
		</div>
	);
};

const PageHeaderTitle = ({
	children,
	className,
	...props
}: React.HTMLAttributes<HTMLHeadingElement>) => {
	return (
		<h2 className={cn("font-heading pb-2 text-3xl", className)} {...props}>
			{children}
		</h2>
	);
};

const PageHeaderDescription = ({
	children,
	className,
	...props
}: React.HTMLAttributes<HTMLParagraphElement>) => {
	return (
		<p className={cn("text-sm text-gray-500", className)} {...props}>
			{children}
		</p>
	);
};

const PageContainer = ({
	children,
	className,
	...props
}: React.HTMLAttributes<HTMLDivElement>) => {
	return (
		<div className={cn("flex flex-col h-full", className)} {...props}>
			<div className="px-32 pt-8 h-full">{children}</div>
		</div>
	);
};

const PageBody = ({
	children,
	className,
	...props
}: React.HTMLAttributes<HTMLDivElement>) => {
	return (
		<div className={cn("flex flex-col h-full", className)} {...props}>
			{children}
		</div>
	);
};

export const Page = {
	Container: PageContainer,
	Header: PageHeader,
	Title: PageHeaderTitle,
	Description: PageHeaderDescription,
	Body: PageBody,
};
