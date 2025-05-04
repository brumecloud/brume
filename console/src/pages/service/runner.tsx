import type { RunnerFragmentFragment } from "@/_apollo/graphql";
import {
  Alert,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useUpdateRunner } from "@/hooks/useUpdateRunner";
import { ServiceFragment } from "@/pages/services";
import type { RouteParams } from "@/router/router.param";
import { DockerRunnerSchema } from "@/schemas/service.schema";
import { cn } from "@/utils";
import { useFragment } from "@apollo/client";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Activity,
  HardDrive,
  Loader2,
  Network,
  RocketIcon,
  SquareTerminal,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useForm, Form } from "react-hook-form";
import { useBlocker, useParams } from "react-router-dom";
import { toast } from "sonner";

export const RunnerPage = () => {
  const { serviceId } = useParams<RouteParams>();
  const [wasDraft, setWasDraft] = useState(false);

  const { data: service, complete } = useFragment({
    from: `Service:${serviceId}`,
    fragment: ServiceFragment,
  });

  if (!complete) {
    throw new Error("Service not complete");
  }

  const runner = service?.liveRunner;
  const draftRunner = service?.draftRunner;

  const { updateRunnerMutation, loading } = useUpdateRunner();

  const form = useForm<RunnerFragmentFragment>({
    resolver: zodResolver(DockerRunnerSchema),
    mode: "onChange",
    defaultValues: useMemo(() => {
      if (draftRunner) return draftRunner;
      if (runner) return runner;
      return undefined;
    }, [draftRunner, runner]),
  });

  const blocker = useBlocker(() => {
    return form.formState.isDirty;
  });

  if (blocker.state === "blocked") {
    toast.warning("You have unsaved changes");
  }

  useEffect(() => {
    if (service) {
      console.log(service);
      if (!draftRunner && !runner) {
        throw new Error("Runner invalid (no draft or live)");
      }
      if (!draftRunner && runner) {
        form.reset(runner);
        return;
      }
      if (draftRunner) {
        console.log("reset draftRunner", draftRunner);
        form.reset(draftRunner);
      }
    }
  }, [form, serviceId]);

  useEffect(() => {
    if (draftRunner) {
      setWasDraft(true);
    }
    if (wasDraft && !draftRunner && runner) {
      form.reset(runner);
      setWasDraft(false);
    }
  }, [serviceId]);

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

  const submitChanges = async () => {
    if (!service?.id) return;

    await updateRunnerMutation({
      variables: {
        serviceId: service.id,
        input: form.getValues().data,
      },
    });

    toast.success("Runner updated");
    form.reset(form.getValues());
  };

  if (!service) return null;

  return (
    <Form {...form}>
      <div className="flex h-full flex-col px-32 pt-8">
        <div className="relative flex flex-row items-center justify-between">
          <div className="absolute right-0 top-[-150px]">
            {form.formState.isDirty && (
              <div className="flex flex-row items-center space-x-2">
                {loading && (
                  <Loader2 className="h-4 w-4 animate-spin" />
                )}
                <Button
                  onClick={submitChanges}
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
        <div className="flex h-full flex-col pt-3">
          <div className="relative flex max-w-[700px] flex-col space-y-4 border-l border-gray-300 pb-16 pl-4">
            <div className="flex flex-row items-center">
              <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
                <SquareTerminal className="h-5 w-5" />
              </div>
              <div className="pl-4">Command</div>
            </div>
            <p className="text-sm font-medium">
              Custom start command
            </p>
            <p className="text-sm">
              Define the command used to start up the service.
            </p>
            <FormField
              control={form.control}
              name="data.command"
              render={({ field, fieldState }) => (
                <div>
                  <Input
                    {...field}
                    placeholder="npx run start"
                    className={cn(
                      "w-[300px] font-mono",
                      runner &&
                        draftRunner &&
                        draftRunner.data.command !==
                          runner.data.command &&
                        "border-blue-500",
                      fieldState.isDirty && "border-green-500"
                    )}
                  />
                  {runner &&
                    draftRunner &&
                    draftRunner.data.command !==
                      runner.data.command && (
                      <p className="text-xs italic text-blue-500">
                        old value: {runner.data.command}
                      </p>
                    )}
                </div>
              )}
            />
          </div>
          <div className="relative flex max-w-[700px] flex-col space-y-4 border-l border-gray-300 pb-16 pl-4">
            <div className="flex flex-row items-center">
              <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
                <Activity className="h-5 w-5" />
              </div>
              <div className="pl-4">Status</div>
            </div>
            <Alert
              variant="default"
              className="border-green-400 bg-green-50">
              <RocketIcon className="h-4 w-4" color="green" />
              <AlertTitle className="text-green-700">
                Service Healthy
              </AlertTitle>
              <AlertDescription className="text-green-800">
                Last health check showed that the service is healthy
              </AlertDescription>
            </Alert>
            <p className="text-sm font-medium">Healthcheck</p>
            <p className="text-sm">
              To be counted as a success the endpoint must return a{" "}
              <span className="font-mono">OK 200</span> status
            </p>
            <FormField
              control={form.control}
              name="data.healthCheckURL"
              render={({ field, fieldState }) => (
                <div className="center flex flex-col gap-2">
                  <Input
                    {...field}
                    placeholder="http://localhost:8080/healthz"
                    className={cn(
                      "w-[300px]",
                      runner &&
                        draftRunner &&
                        draftRunner.data.healthCheckURL !==
                          runner.data.healthCheckURL &&
                        "border-blue-500",
                      fieldState.isDirty && "border-green-500",
                      fieldState.error && "border-red-500"
                    )}
                  />
                  {runner &&
                    draftRunner &&
                    draftRunner.data.healthCheckURL !==
                      runner.data.healthCheckURL && (
                      <p className="text-xs italic text-blue-500">
                        old value: {runner.data.healthCheckURL}
                      </p>
                    )}
                  {fieldState.error && (
                    <p className="text-xs text-red-500">
                      {fieldState.error.message}
                    </p>
                  )}
                </div>
              )}
            />
          </div>
          <div className="relative flex flex-col space-y-4 border-l border-gray-300 pb-16 pl-4">
            <div className="flex flex-row items-center">
              <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
                <HardDrive className="h-5 w-5" />
              </div>
              <div className="pl-4">Ressources</div>
            </div>
            <div className="text-sm">
              Define limits & request for the service
            </div>
            <div className="flex flex-col gap-1">
              <p className="text-sm font-medium">Request</p>
              <p className="text-sm italic">
                This number represents what Brume needs to allocated
                in minimum for the service
              </p>
              <div className="flex flex-row gap-3">
                <div>
                  <Label htmlFor="request-cpu" className="text-xs">
                    CPU
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.cpu.request"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          id="request-cpu"
                          type="number"
                          step={0.1}
                          placeholder="0.1CPU"
                          className={cn(
                            "w-[200px]",
                            runner &&
                              draftRunner &&
                              draftRunner.data.cpu.request !==
                                runner.data.cpu.request &&
                              "border-blue-500",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                        {fieldState.error && (
                          <p className="text-xs text-red-500">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.cpu.request !==
                            runner.data.cpu.request && (
                            <p className="text-xs italic text-blue-500">
                              old value: {runner.data.cpu.request}
                            </p>
                          )}
                      </div>
                    )}
                  />
                </div>
                <div>
                  <Label htmlFor="request-cpu" className="text-xs">
                    Memory
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.memory.request"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          id="request-cpu"
                          type="number"
                          step={100}
                          placeholder="100Mb"
                          className={cn(
                            "w-[200px]",
                            runner &&
                              draftRunner &&
                              draftRunner.data.memory.request !==
                                runner.data.memory.request &&
                              "border-blue-500",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                        {fieldState.error && (
                          <p className="text-xs text-red-500">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.memory.request !==
                            runner.data.memory.request && (
                            <p className="text-xs italic text-blue-500">
                              old value: {runner.data.memory.request}
                            </p>
                          )}
                      </div>
                    )}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-col gap-1">
              <p className="text-sm font-medium">Limits</p>
              <p className="text-sm italic">
                This number represents what Brume needs to allocated
                in <span className="font-medium">maximum</span> for
                this service. If the value is exceded, the service
                will be stopped
              </p>
              <div className="flex flex-row gap-3">
                <div>
                  <Label htmlFor="request-cpu" className="text-xs">
                    CPU
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.cpu.limit"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          id="request-cpu"
                          type="number"
                          step={0.1}
                          placeholder="0.2CPU"
                          className={cn(
                            "w-[200px]",
                            runner &&
                              draftRunner &&
                              draftRunner.data.cpu.limit !==
                                runner.data.cpu.limit &&
                              "border-blue-500",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                        {fieldState.error && (
                          <p className="text-xs text-red-500">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.cpu.limit !==
                            runner.data.cpu.limit && (
                            <p className="text-xs italic text-blue-500">
                              old value: {runner.data.cpu.limit}
                            </p>
                          )}
                      </div>
                    )}
                  />
                </div>
                <div>
                  <Label htmlFor="request-cpu" className="text-xs">
                    Memory
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.memory.limit"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          id="request-cpu"
                          type="number"
                          step={100}
                          placeholder="150Mb"
                          className={cn(
                            "w-[200px]",
                            runner &&
                              draftRunner &&
                              draftRunner.data.memory.limit !==
                                runner.data.memory.limit &&
                              "border-blue-500",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                        {fieldState.error && (
                          <p className="text-xs text-red-500">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.memory.limit !==
                            runner.data.memory.limit && (
                            <p className="text-xs italic text-blue-500">
                              old value: {runner.data.memory.limit}
                            </p>
                          )}
                      </div>
                    )}
                  />
                </div>
              </div>
            </div>
          </div>
          <div className="relative flex flex-col space-y-4 border-l border-gray-300 pb-16 pl-4">
            <div className="flex flex-row items-center">
              <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
                <Network className="h-5 w-5" />
              </div>
              <div className="pl-4">Network</div>
            </div>
            <div className="flex flex-col gap-3">
              <div className="flex flex-col gap-1">
                <p className="text-sm font-medium">
                  Public networking
                </p>
                <p className="text-sm">
                  Access your application over HTTPS with the
                  following domains.
                </p>
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="domain" className="text-xs">
                  Domain
                </Label>
                <FormField
                  control={form.control}
                  name="data.publicDomain"
                  render={({ field, fieldState }) => (
                    <div className="flex flex-col gap-2">
                      <div className="relative w-[300px]">
                        <Input
                          {...field}
                          id="domain"
                          type="text"
                          placeholder="my-app.brume.run"
                          className={cn(
                            "w-[300px]",
                            runner &&
                              draftRunner &&
                              draftRunner.data.publicDomain !==
                                runner.data.publicDomain &&
                              "border-blue-500",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                        <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                          <span className="text-sm text-muted-foreground">
                            .brume.run
                          </span>
                        </div>
                      </div>
                      {fieldState.error && (
                        <p className="text-xs text-red-500">
                          {fieldState.error.message}
                        </p>
                      )}
                      {runner &&
                        draftRunner &&
                        draftRunner.data.publicDomain !==
                          runner.data.publicDomain && (
                          <p className="text-xs italic text-blue-500">
                            old value: {runner.data.publicDomain}
                          </p>
                        )}
                    </div>
                  )}
                />
              </div>
            </div>
            <div className="flex flex-col gap-1">
              <div className="flex flex-col gap-1">
                <p className="text-sm font-medium">
                  Private networking
                </p>
                <p className="text-sm">
                  Communicate with other services inside Brume private
                  network.
                </p>
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="domain" className="text-xs">
                  Private domain
                </Label>
                <FormField
                  control={form.control}
                  name="data.privateDomain"
                  render={({ field, fieldState }) => (
                    <div className="flex flex-col gap-2">
                      <div className="relative w-[300px]">
                        <Input
                          {...field}
                          type="text"
                          placeholder="chat"
                          className={cn(
                            "w-[300px]",
                            runner &&
                              draftRunner &&
                              draftRunner.data.privateDomain !==
                                runner.data.privateDomain &&
                              "border-blue-500",
                            fieldState.isDirty && "border-green-500",
                            fieldState.error && "border-red-500"
                          )}
                        />
                        <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                          <span className="text-sm text-muted-foreground">
                            .brume.internal
                          </span>
                        </div>
                      </div>
                      {fieldState.error && (
                        <p className="text-xs text-red-500">
                          {fieldState.error.message}
                        </p>
                      )}
                      {runner &&
                        draftRunner &&
                        draftRunner.data.privateDomain !==
                          runner.data.privateDomain && (
                          <p className="text-xs italic text-blue-500">
                            old value: {runner.data.privateDomain}
                          </p>
                        )}
                    </div>
                  )}
                />
              </div>
            </div>
          </div>
          <div className="grow border-l border-gray-300" />
        </div>
      </div>
    </Form>
  );
};
