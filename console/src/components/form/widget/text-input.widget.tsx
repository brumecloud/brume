import type { WidgetProps } from "@rjsf/utils";
import { Input } from "@/components/ui/input";

export const TextInputWidget = (props: WidgetProps) => {
  const { id, value, disabled, onChange, placeholder } = props;

  return (
    <Input
      className="max-w-[400px]"
      disabled={disabled}
      id={id}
      onChange={onChange}
      placeholder={placeholder}
      value={value || ""}
    />
  );
};
