import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input, type InputProps } from "@/components/ui/input";
import { cn } from "@/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { Plus, PlusCircle, Trash } from "lucide-react";
import React, { useState } from "react";
import { Form, useFieldArray, useForm } from "react-hook-form";
import { z } from "zod";

export const VariableSchema = z.object({
  variables: z.array(
    z.object({
      key: z
        .string()
        .min(1)
        .max(30)
        .regex(/^[a-zA-Z0-9_]+$/),
      value: z.string().min(1),
    })
  ),
});

type Variable = z.infer<typeof VariableSchema>;

const HiddableInput = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, ...props }, ref) => {
    const [isVisible, setIsVisible] = useState(false);
    const [isFocused, setIsFocused] = useState(false);

    return (
      <Input
        className={cn(className)}
        ref={ref}
        {...props}
        type={isVisible || isFocused ? "text" : "password"}
        onMouseEnter={() => setIsVisible(true)}
        onMouseLeave={() => setIsVisible(false)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
      />
    );
  }
);

export const VariablesPage = () => {
  const form = useForm<Variable>({
    resolver: zodResolver(VariableSchema),
    mode: "onChange",
    defaultValues: {
      variables: [
        { key: "FOO", value: "BAR" },
        { key: "BAZ", value: "QUX" },
      ],
    },
  });

  const { fields, append, remove } = useFieldArray({
    control: form.control,
    name: "variables",
  });

  return (
    <div className="flex h-full flex-col px-32 pt-8">
      <div className="center flex flex-col gap-2 space-y-2">
        <h3 className="text-md font-medium">
          Change the service variables
        </h3>
        <p className="text-sm">
          Variables can also be defined in the project settings (this
          variables will be global to the project)
        </p>
        <div>
          {form.formState.isDirty && (
            <div className="flex flex-row items-center space-x-2">
              {/* {loading && (
                <Loader2 className="h-4 w-4 animate-spin" />
              )} */}
              <Button
                variant="outline"
                disabled={
                  Object.keys(form.formState.errors).length > 0
                }
                size="sm"
                className="text-xs">
                Save changes
              </Button>
              <Button
                onClick={() => form.reset()}
                variant="destructive"
                size="sm"
                className="text-xs">
                Discard
              </Button>
            </div>
          )}
        </div>
      </div>
      <div className="mt-4 flex flex-col gap-4">
        <Form {...form}>
          <div className="flex flex-col gap-2">
            {fields.map((item, index) => (
              <div className="flex flex-row items-center gap-2">
                <div className="grid w-full grid-cols-10 items-center gap-x-2">
                  <div className="col-span-5">
                    <FormField
                      control={form.control}
                      name={`variables.${index}.key`}
                      render={({ field, fieldState }) => (
                        <Input
                          {...field}
                          placeholder="e. g. CLIENT_KEY"
                          className={cn(
                            "shadow-none",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                      )}
                    />
                  </div>
                  <div className="col-span-5">
                    <FormField
                      control={form.control}
                      name={`variables.${index}.value`}
                      render={({ field, fieldState }) => (
                        <HiddableInput
                          {...field}
                          className={cn(
                            "shadow-none",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                      )}
                    />
                  </div>
                </div>
                <div className="flex justify-end">
                  <Button
                    variant="ghost"
                    onClick={() => remove(index)}>
                    <Trash className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
          <Button
            variant="outline"
            className="mt-4"
            onClick={() => append({ key: "", value: "" })}>
            <PlusCircle className="mr-2 h-4 w-4" />
            Add Variable
          </Button>
        </Form>{" "}
      </div>
    </div>
  );
};
