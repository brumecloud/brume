import { Switch } from "~/components/ui/switch";

import { SettingField } from "./setting-field";

interface SwitchSettingProps {
  readonly label: string;
  readonly description: string;
  readonly text: string;
  readonly checked: boolean;
  readonly staged: boolean;
  readonly onCommit: (checked: boolean) => void;
}

export function SwitchSetting({
  label,
  description,
  text,
  checked,
  staged,
  onCommit,
}: SwitchSettingProps) {
  return (
    <SettingField label={label} description={description} staged={staged}>
      <label className="flex items-center gap-3 rounded-lg border bg-background px-4 py-3 text-sm">
        <Switch checked={checked} onCheckedChange={onCommit} />
        {text}
      </label>
    </SettingField>
  );
}
