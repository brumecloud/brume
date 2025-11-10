import type { FieldTemplateProps } from "@rjsf/utils";
import { Label } from "@/components/ui/label";

export const FieldTemplate = (props: FieldTemplateProps) => {
  const { id, required, label, help, description, errors, children } = props;
  const isObjectField = props.schema.type === "object";

  if (isObjectField) {
    return children;
  }

  return (
    <div className="flex flex-col gap-1">
      <div className="pb-2">
        <Label className="" htmlFor={id}>
          {label}
          {required && <span className="text-destructive">*</span>}
        </Label>
        {description && (
          <p className="text-muted-foreground text-xs">{description}</p>
        )}
      </div>
      {children}
      {errors && <p className="text-destructive text-xs">{errors}</p>}
      {help && <p className="text-muted-foreground text-xs">{help}</p>}
    </div>
  );
};
