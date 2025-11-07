import { VariableForm } from "@/components/form/variable.form";

export const VariablesPage = () => (
  <div className="px-32 pt-8">
    <VariableForm
      heading="Change the service variables"
      description="Variables can also be defined in the service settings (this variables will be global to the project and will be inherited by the services)"
    />
  </div>
);
