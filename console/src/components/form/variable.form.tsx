import { zodResolver } from "@hookform/resolvers/zod";
import { PlusCircle, Trash } from "lucide-react";
import React, { useState } from "react";
import { useFieldArray, useForm } from "react-hook-form";
import { Form } from "react-router-dom";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input, type InputProps } from "@/components/ui/input";
import { cn } from "@/utils";

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
        onBlur={() => setIsFocused(false)}
        onFocus={() => setIsFocused(true)}
        onMouseEnter={() => setIsVisible(true)}
        onMouseLeave={() => setIsVisible(false)}
        type={isVisible || isFocused ? "text" : "password"}
      />
    );
  }
);

export const VariableForm = ({
  heading,
  description,
}: {
  heading?: string;
  description?: string;
}) => {
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
    <div className="flex h-full flex-col">
      <div className="center flex flex-col gap-2 space-y-2">
        {heading && (
          <>
            <h3 className="font-medium text-md">{heading}</h3>
            <p className="text-sm">{description}</p>
          </>
        )}
        <div>
          {form.formState.isDirty && (
            <div className="flex flex-row items-center space-x-2">
              {/* {loading && (
                <Loader2 className="h-4 w-4 animate-spin" />
              )} */}
              <Button
                className="text-xs"
                disabled={Object.keys(form.formState.errors).length > 0}
                size="sm"
                variant="outline"
              >
                Save changes
              </Button>
              <Button
                className="text-xs"
                onClick={() => form.reset()}
                size="sm"
                variant="destructive"
              >
                Discard
              </Button>
            </div>
          )}
        </div>
      </div>
      <div className="mt-4 flex flex-col gap-4">
        <Form {...form}>
          <div className="flex flex-col gap-2">
            {fields.map((_item, index) => (
              <div className="flex flex-row items-center gap-2">
                <div className="grid w-full grid-cols-10 items-center gap-x-2">
                  <div className="col-span-5">
                    <FormField
                      control={form.control}
                      name={`variables.${index}.key`}
                      render={({ field, fieldState }) => (
                        <Input
                          {...field}
                          className={cn(
                            "shadow-none",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                          placeholder="e. g. CLIENT_KEY"
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
                  <Button onClick={() => remove(index)} variant="ghost">
                    <Trash className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
          <Button
            className="mt-4"
            onClick={() => append({ key: "", value: "" })}
            variant="outline"
          >
            <PlusCircle className="mr-2 h-4 w-4" />
            Add Variable
          </Button>
        </Form>{" "}
      </div>
    </div>
  );
};
