import type { WidgetProps } from "@rjsf/utils";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";

export const CheckboxWidget = (props: WidgetProps) => {
  const { id, value, disabled, onChange, schema } = props;

  return (
    <div className="flex items-center gap-2">
      <Checkbox
        disabled={disabled}
        id={id}
        onCheckedChange={onChange}
        value={value}
      />
      <Label className="font-normal" htmlFor={id}>
        {schema.label}
      </Label>
    </div>
  );
};
