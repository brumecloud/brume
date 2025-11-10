import type { WidgetProps } from "@rjsf/utils";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export const SelectWidget = (props: WidgetProps) => {
  const { value, disabled, onChange, schema } = props;

  return (
    <Select disabled={disabled} onValueChange={onChange} value={value}>
      <SelectTrigger className="w-[400px]">
        <SelectValue placeholder={schema.placeholder || "Select a value"} />
      </SelectTrigger>
      <SelectContent>
        {schema.enum.map((option) => (
          <SelectItem key={option} value={option}>
            {option}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};
