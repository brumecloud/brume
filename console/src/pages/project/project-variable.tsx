import { VariableForm } from "@/components/form/variable.form";

export function ProjectVariable() {
  return (
    <div className="flex flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row items-center justify-between pt-16">
          <div className="flex flex-col pb-8">
            <h2 className="pb-2 font-heading text-3xl">Variables</h2>
            <p>
              Define variables which will be used by all services in
              the project. This variables will be injected in the
              build and run phase of your services.
            </p>
          </div>
        </div>
        <VariableForm />
      </div>
    </div>
  );
}
