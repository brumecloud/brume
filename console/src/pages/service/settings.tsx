import { useFragment } from "@apollo/client";
import { zodResolver } from "@hookform/resolvers/zod";
import { Flame, SquareTerminal } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { useBlocker, useNavigate, useParams } from "react-router-dom";
import { toast } from "sonner";
import { z } from "zod";
import type { ServiceFragmentFragment } from "@/_apollo/graphql";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { ServiceFragment } from "@/gql/service.graphql";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

export const SettingPage = () => {
  const { serviceId } = useParams<RouteParams>();

  const { data: service, complete } = useFragment({
    from: `Service:${serviceId}`,
    fragment: ServiceFragment,
    fragmentName: "ServiceFragment",
  });

  if (!complete) {
    throw new Error("Service not complete");
  }

  const [_confirmModalOpen, setConfirmModalOpen] = useState(false);
  const navigate = useNavigate();

  const form = useForm<ServiceFragmentFragment>({
    resolver: zodResolver(
      z.object({
        name: z.string(),
      })
    ),
    mode: "onChange",
    defaultValues: service,
  });

  const blocker = useBlocker(() => form.formState.isDirty);

  if (blocker.state === "blocked") {
    toast.warning("You have unsaved changes");
  }

  useEffect(() => {
    if (service) {
      form.reset(service);
    }
  }, [service, form.reset]);

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
    <Page.Container>
      <Page.Body className="pt-4">
        <Stepper.Root leftBorder>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <SquareTerminal className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>General</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {() => (
                <>
                  <p className="text-sm">Define the name of the service</p>
                  <FormField
                    control={form.control}
                    name="name"
                    render={({ field, fieldState }) => (
                      <Input
                        {...field}
                        className={cn(
                          "w-[400px]",
                          fieldState.isDirty && "border-green-500"
                        )}
                        placeholder="My Service"
                      />
                    )}
                  />
                  <p className="text-sm">The universal ID of the service</p>
                  <Input className="w-[400px]" disabled value={service?.id} />
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item className="h-full">
            <Stepper.Header>
              <Stepper.Icon className="border-red-300 bg-red-50">
                <Flame className="h-5 w-5 text-red-900" />
              </Stepper.Icon>
              <Stepper.Title className="text-red-800">
                Danger zone
              </Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {() => (
                <>
                  <div className="w-1/2 rounded-md border border-red-200 bg-red-50 p-3 text-red-800">
                    <p>
                      Deleting the service will delete all data associated to it
                      :{" "}
                      <span className="font-semibold">
                        all its artifacts, all its logs and metrics.
                      </span>
                    </p>
                  </div>
                  <Button
                    className="w-[100px] bg-red-700 hover:bg-red-800"
                    onClick={() => setConfirmModalOpen(true)}
                  >
                    Delete
                  </Button>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
        </Stepper.Root>
      </Page.Body>
    </Page.Container>
  );
};
