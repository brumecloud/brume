import Form from "@rjsf/core";
import type { RegistryWidgetsType, RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";
import { FieldTemplate } from "./template/field.template";
import { ObjectFieldTemplate } from "./template/object-field.template";
import { CheckboxWidget } from "./widget/checkbox.widget";
import { SelectWidget } from "./widget/select.widget";
import { TextInputWidget } from "./widget/text-input.widget";

type BrumeFormProps = {
  schema: RJSFSchema;
  uiSchema: RJSFSchema;
};

const Widgets: RegistryWidgetsType = {
  TextWidget: TextInputWidget,
  CheckboxWidget,
  SelectWidget,
};

export const BrumeForm = ({ schema, uiSchema }: BrumeFormProps) => (
  <Form
    schema={schema}
    templates={{
      FieldTemplate,
      ObjectFieldTemplate,
    }}
    uiSchema={uiSchema}
    validator={validator}
    widgets={Widgets}
  >
    <button className="hidden" type="submit" />
  </Form>
);
