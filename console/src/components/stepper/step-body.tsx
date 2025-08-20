import { MotionConfig } from "motion/react";
import * as React from "react";
import { cn } from "@/utils";

export const StepContext = React.createContext<{
	step: number;
	advance: () => void;
	rewind: () => void;
	setStep: (step: number) => void;
}>({
	step: 0,
	advance: () => {},
	rewind: () => {},
	setStep: () => {},
});

const StepItemContext = React.createContext<{
	step: number;
	active: boolean;
	toggleActive: () => void;
}>({
	step: 0,
	toggleActive: () => {},
	active: false,
});

export const useStepItem = () => {
	const context = React.useContext(StepItemContext);

	if (!context) {
		throw new Error("useStepItem must be used within a <StepItem />");
	}

	return context;
};

export const useStep = () => {
	const context = React.useContext(StepContext);

	if (!context) {
		throw new Error("useStep must be used within a <StepRoot />");
	}

	return context;
};

export const StepRoot = ({
	children,
	className,
	...props
}: React.HTMLAttributes<HTMLDivElement>) => {
	const [step, setStep] = React.useState(0);
	const scrollRef = React.useRef<HTMLDivElement>(null);

	const [activeStep, setActiveStep] = React.useState(
		Array.from({ length: React.Children.count(children) }, () => false),
	);

	const advance = () => {
		setStep(step + 1);
	};

	const rewind = () => {
		setStep(step - 1);
	};

	const setStepFn = (step: number) => {
		if (step < 0) {
			rewind();
		} else if (step > React.Children.count(children) - 1) {
			advance();
		} else {
			setStep(step);
		}
	};

	const toggleActive = (index: number) => {
		setActiveStep((prev) => {
			const newActiveStep = [...prev];
			newActiveStep[index] = !newActiveStep[index];
			return newActiveStep;
		});
	};

	React.useEffect(() => {
		if (step === React.Children.count(children)) {
			setTimeout(() => {
				for (let i = React.Children.count(children) - 1; i >= 0; i--) {
					setTimeout(
						() => {
							toggleActive(i);
						},
						(React.Children.count(children) - i) * 100,
					);
				}
			}, 800);
		}
	}, [step]);

	return (
		<StepContext.Provider value={{ step, advance, rewind, setStep: setStepFn }}>
			<MotionConfig transition={{ duration: 0.3 }}>
				<div className={cn("flex flex-col h-full", className)} {...props}>
					{React.Children.map(children, (child, index) => {
						return (
							<StepItemContext.Provider
								value={{
									step: index,
									active: activeStep[index] as boolean,
									toggleActive: () => toggleActive(index),
								}}
							>
								{child}
							</StepItemContext.Provider>
						);
					})}
					<div ref={scrollRef} />
				</div>
			</MotionConfig>
		</StepContext.Provider>
	);
};
