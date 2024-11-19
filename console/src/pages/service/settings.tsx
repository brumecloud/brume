import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  RadioGroup,
  RadioGroupItem,
} from "@/components/ui/radio-group";
import { useService } from "@/hooks/useService";
import { type Service } from "@/schemas/service.schema";
import { cn } from "@/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { Bell, Flame, Pickaxe, SquareTerminal } from "lucide-react";
import { useCallback } from "react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useBlocker } from "react-router-dom";
import { toast } from "sonner";
import { z } from "zod";

export const SettingPage = () => {
  const { service } = useService();

  const form = useForm<Service>({
    resolver: zodResolver(
      z.object({
        name: z.string(),
      })
    ),
    mode: "onChange",
    defaultValues: service,
  });

  const blocker = useBlocker(() => {
    return form.formState.isDirty;
  });

  if (blocker.state === "blocked") {
    toast.warning("You have unsaved changes");
  }

  useEffect(() => {
    if (service) {
      form.reset(service);
    }
  }, [service?.__typename, service?.id]);

  const onUnload = useCallback(
    (e: BeforeUnloadEvent) => {
      if (form.formState.isDirty) {
        e.preventDefault();
      }
    },
    [form.formState.isDirty]
  );

  useEffect(() => {
    window.addEventListener("beforeunload", onUnload);
    return () => window.removeEventListener("beforeunload", onUnload);
  }, [onUnload]);

  return (
    <div className="flex h-full flex-col px-32 pt-8">
      <div className="flex flex-col py-16">
        <h2 className="h-full pb-2 text-2xl font-semibold">
          Settings
        </h2>
        <p>Manage the service</p>
      </div>
      <div className="flex h-full flex-col">
        <div className="relative flex max-w-[700px] flex-col space-y-4 border-l border-gray-300 pb-16 pl-4">
          <div className="flex flex-row items-center">
            <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
              <SquareTerminal className="h-5 w-5" />
            </div>
            <div className="pl-4">General</div>
          </div>
          <p className="text-sm">Define the name of the service</p>
          <FormField
            control={form.control}
            name="name"
            render={({ field, fieldState }) => (
              <Input
                {...field}
                placeholder="My Service"
                className={cn(
                  "w-[400px]",
                  fieldState.isDirty && "border-green-500"
                )}
              />
            )}
          />
          <p className="text-sm">The universal ID of the service</p>
          <Input value={service?.id} disabled className="w-[400px]" />
        </div>
        <div className="relative flex flex-col border-l border-gray-300 pl-4">
          <div className="flex flex-row items-center">
            <div className="absolute left-[-20px] rounded-full border border-red-300 bg-white p-2 text-red-600">
              <Flame className="h-5 w-5" />
            </div>
            <div className="pl-4 text-red-800">Danger zone</div>
          </div>
          <div className="flex flex-col space-y-6 pt-4 text-red-900">
            <div>
              <p>
                Deleting the service will delete all data associated
                to it :{" "}
                <span className="font-semibold">
                  all its artifacts, all its logs and metrics.
                </span>
              </p>
              <p>This cannot be undone.</p>
            </div>
            <Button className="w-[100px] bg-red-700">Delete</Button>
          </div>
        </div>
        <div className="grow border-l border-gray-300" />
      </div>
    </div>
  );
};
