import { AnimatePresence, motion } from "motion/react";
import type React from "react";
import { cn } from "@/utils";
import { useStep, useStepItem } from "./step-body";

export const StepItem = ({
	children,
	className,
	...props
}: React.HTMLAttributes<HTMLDivElement>) => {
	const { step: currentStep } = useStep();
	const { step } = useStepItem();
	const isValidated = currentStep > step;

	return (
		<div
			className={cn(
				"flex flex-col gap-4 transition-all duration-300 relative border-l pb-8 border-gray-300 pl-4",
				isValidated && "border-green-500",
				className,
			)}
			{...props}
		>
			{children}
		</div>
	);
};

export const StepHeader = ({ children }: { children: React.ReactNode }) => {
	const { step: currentStep } = useStep();
	const { step, toggleActive } = useStepItem();
	const canAdvance = step < currentStep;

	return (
		<button
			type="button"
			className={cn("flex flex-row items-center")}
			disabled={!canAdvance}
			onClick={() => toggleActive()}
		>
			{children}
		</button>
	);
};

export const StepHeaderTitle = ({
	children,
}: {
	children: React.ReactNode;
}) => {
	return <div className="text-lg font-medium pl-4">{children}</div>;
};

export const StepIcon = ({ children }: { children: React.ReactNode }) => {
	const { step: currentStep } = useStep();
	const { step } = useStepItem();
	const isValidated = currentStep > step;

	return (
		<div
			className={cn(
				"absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600 transition-all duration-300",
				isValidated && "border-green-500 text-green-600",
			)}
		>
			{children}
		</div>
	);
};

export const StepBody = ({
	children,
}: {
	children: ({
		step,
		setStep,
		advance,
		rewind,
	}: {
		step: number;
		setStep: (step: number) => void;
		advance: () => void;
		rewind: () => void;
	}) => React.ReactNode;
}) => {
	const { step: currentStep, advance, rewind, setStep } = useStep();
	const { step, active } = useStepItem();

	const isActive = currentStep == step || active;

	return (
		<div className="min-h-2">
			<AnimatePresence>
				{isActive && (
					<motion.div
						initial={{ opacity: 0, height: 0 }}
						animate={{ opacity: 1, height: "auto" }}
						exit={{ opacity: 0, height: 0 }}
						className="flex flex-col gap-4 overflow-hidden"
					>
						{children({ step: currentStep, setStep, advance, rewind })}
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	);
};
