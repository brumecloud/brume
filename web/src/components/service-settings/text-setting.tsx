import { useState } from "react";

import type { SettingValue } from "~/api/types";
import { Input } from "~/components/ui/input";
import { validateSetting } from "~/helpers/changes";

import { SettingField } from "./setting-field";

interface TextSettingProps {
  readonly field: Parameters<typeof validateSetting>[0];
  readonly label: string;
  readonly description?: string;
  // The value after the staged change, if any. Give the component this value as `key` too:
  // a deploy or a discard then resets what was typed.
  readonly value: string;
  readonly staged: boolean;
  readonly placeholder?: string;
  readonly onCommit: (value: SettingValue) => void;
}

// The draft is staged when the field loses focus or on Enter, once valid.
export function TextSetting({
  field,
  label,
  description,
  value,
  staged,
  placeholder,
  onCommit,
}: TextSettingProps) {
  const [text, setText] = useState(value);
  const [error, setError] = useState<string>();

  const commit = () => {
    const result = validateSetting(field, text);
    setError("error" in result ? result.error : undefined);
    if ("value" in result) {
      onCommit(result.value);
    }
  };

  return (
    <SettingField
      label={label}
      {...(description ? { description } : {})}
      staged={staged}
      error={error}
    >
      <Input
        value={text}
        aria-label={label}
        aria-invalid={error ? true : undefined}
        placeholder={placeholder}
        onChange={(event) => setText(event.target.value)}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === "Enter" && !event.shiftKey) {
            commit();
          }
        }}
      />
    </SettingField>
  );
}
