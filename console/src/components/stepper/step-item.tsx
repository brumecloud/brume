import { cn } from "@/utils";
import { AnimatePresence, motion } from "motion/react";
import type React from "react";

import { useStep, useStepItem } from "./step-body";

export const StepItem = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => {
  const { step: currentStep, leftBorder } = useStep();
  const { step } = useStepItem();
  const isValidated = currentStep > step;

  return (
    <div
      className={cn(
        "relative flex flex-col gap-4 pb-8 pl-4 transition-all duration-300",
        className
      )}
      {...props}>
      {leftBorder && (
        <div className="absolute left-[-1px] top-0 h-full w-[1px] bg-gray-300" />
      )}
      {isValidated && leftBorder && (
        <motion.div
          initial={{ height: 0 }}
          animate={{ height: "100%" }}
          exit={{ height: 0 }}
          transition={{ duration: 0.5 }}
          className="absolute left-[-1px] top-0 w-[1px] bg-green-500"
        />
      )}
      {children}
    </div>
  );
};

export const StepHeader = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLButtonElement>) => {
  const { step: currentStep } = useStep();
  const { step, toggleActive } = useStepItem();
  const canAdvance = step < currentStep;

  return (
    <button
      type="button"
      className={cn("flex flex-row items-center", className)}
      disabled={!canAdvance}
      onClick={() => toggleActive()}
      {...props}>
      {children}
    </button>
  );
};

export const StepHeaderTitle = ({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => {
  return (
    <div
      className={cn("pl-4 text-lg font-medium", className)}
      {...props}>
      {children}
    </div>
  );
};

export const StepIcon = ({
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
        "absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600 transition-all duration-300",
        isValidated && "border-green-500 text-green-600",
        className
      )}
      {...props}>
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
  const {
    step: currentStep,
    advance,
    rewind,
    setStep,
    shouldAnimate,
  } = useStep();
  const { step, active } = useStepItem();

  const isActive = currentStep === step || active || !shouldAnimate;

  return (
    <div className="min-h-2">
      <AnimatePresence>
        {isActive && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            transition={{
              duration: shouldAnimate ? 0.3 : 0.5,
            }}
            className="flex flex-col gap-4 overflow-hidden">
            {children({
              step: currentStep,
              setStep,
              advance,
              rewind,
            })}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};
