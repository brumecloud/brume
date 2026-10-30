import { useState } from "react";

import { Slider } from "~/components/ui/slider";
import { limitStep } from "~/helpers/changes";

import { SettingField } from "./setting-field";

interface LimitSettingProps {
  readonly label: string;
  readonly steps: readonly number[];
  readonly value: number;
  readonly staged: boolean;
  readonly format: (value: number) => string;
  readonly onCommit: (value: number) => void;
}

// Reference: infra/screens/04e-service-settings-4-scale.png. The slider moves over fixed steps;
// the value is staged when the admin lets go of it.
export function LimitSetting({ label, steps, value, staged, format, onCommit }: LimitSettingProps) {
  const [index, setIndex] = useState(limitStep(steps, value));
  const shown = steps[index] ?? value;

  return (
    <SettingField label={label} staged={staged}>
      <div className="flex flex-col gap-3 rounded-lg border bg-background px-4 py-3">
        <span className="text-sm">
          {label} : <span className="font-medium">{format(shown)}</span>
        </span>
        <Slider
          min={0}
          max={steps.length - 1}
          step={1}
          value={index}
          getAriaLabel={() => label}
          onValueChange={(next) => setIndex(Number(next))}
          onValueCommitted={(next) => onCommit(steps[Number(next)] ?? value)}
        />
      </div>
    </SettingField>
  );
}
