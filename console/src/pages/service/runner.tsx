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
import { Form, useForm } from "react-hook-form";
import { useBlocker, useParams } from "react-router-dom";
import { toast } from "sonner";
import type { RunnerFragmentFragment } from "@/_apollo/graphql";
import { Stepper } from "@/components/stepper";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ServiceFragment } from "@/gql/service.graphql";
import { useUpdateRunner } from "@/hooks/useUpdateRunner";
import type { RouteParams } from "@/router/router.param";
import { DockerRunnerSchema } from "@/schemas/service.schema";
import { cn } from "@/utils";

export const RunnerPage = () => {
  const { serviceId } = useParams<RouteParams>();
  const [wasDraft, setWasDraft] = useState(false);

  const { data: service, complete } = useFragment({
    from: `Service:${serviceId}`,
    fragment: ServiceFragment,
    fragmentName: "ServiceFragment",
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
      if (draftRunner) {
        return draftRunner;
      }
      if (runner) {
        return runner;
      }
      return;
    }, [draftRunner, runner]),
  });

  const blocker = useBlocker(() => form.formState.isDirty);

  if (blocker.state === "blocked") {
    toast.warning("You have unsaved changes");
  }

  useEffect(() => {
    if (service) {
      console.log(service);
      if (!(draftRunner || runner)) {
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
  }, [form, runner, service, draftRunner]);

  useEffect(() => {
    if (draftRunner) {
      setWasDraft(true);
    }
    if (wasDraft && !draftRunner && runner) {
      form.reset(runner);
      setWasDraft(false);
    }
  }, [runner, draftRunner, wasDraft, form.reset]);

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
    if (!service?.id) {
      return;
    }

    await updateRunnerMutation({
      variables: {
        serviceId: service.id,
        input: form.getValues().data,
      },
    });

    toast.success("Runner updated");
    form.reset(form.getValues());
  };

  if (!service) {
    return null;
  }

  return (
    <Form {...form} className="px-32 pt-16">
      <div className="absolute top-[-150px] right-0">
        {form.formState.isDirty && (
          <div className="flex flex-row items-center space-x-2">
            {loading && <Loader2 className="h-4 w-4 animate-spin" />}
            <Button
              className="text-xs"
              disabled={Object.keys(form.formState.errors).length > 0}
              onClick={submitChanges}
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
      <Stepper.Root leftBorder>
        <Stepper.Item>
          <Stepper.Header>
            <Stepper.Icon>
              <SquareTerminal className="h-5 w-5" />
            </Stepper.Icon>
            <Stepper.Title>Command</Stepper.Title>
          </Stepper.Header>
          <Stepper.Body>
            {() => (
              <>
                <p className="font-medium text-sm">Custom start command</p>
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
                        className={cn(
                          "w-[300px] font-mono",
                          runner &&
                            draftRunner &&
                            draftRunner.data.command !== runner.data.command &&
                            "border-blue-500",
                          fieldState.isDirty && "border-green-500"
                        )}
                        placeholder="npx run start"
                      />
                      {runner &&
                        draftRunner &&
                        draftRunner.data.command !== runner.data.command && (
                          <p className="text-blue-500 text-xs italic">
                            old value: {runner.data.command}
                          </p>
                        )}
                    </div>
                  )}
                />
              </>
            )}
          </Stepper.Body>
        </Stepper.Item>
        <Stepper.Item>
          <Stepper.Header>
            <Stepper.Icon>
              <Activity className="h-5 w-5" />
            </Stepper.Icon>
            <Stepper.Title>Status</Stepper.Title>
          </Stepper.Header>
          <Stepper.Body>
            {() => (
              <>
                <Alert
                  className="w-1/2 border-green-400 bg-green-50"
                  variant="default"
                >
                  <RocketIcon className="h-4 w-4" color="green" />
                  <AlertTitle className="text-green-700">
                    Service Healthy
                  </AlertTitle>
                  <AlertDescription className="text-green-800">
                    Last health check showed that the service is healthy
                  </AlertDescription>
                </Alert>
                <p className="font-medium text-sm">Healthcheck</p>
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
                        placeholder="http://localhost:8080/healthz"
                      />
                      {runner &&
                        draftRunner &&
                        draftRunner.data.healthCheckURL !==
                          runner.data.healthCheckURL && (
                          <p className="text-blue-500 text-xs italic">
                            old value: {runner.data.healthCheckURL}
                          </p>
                        )}
                      {fieldState.error && (
                        <p className="text-red-500 text-xs">
                          {fieldState.error.message}
                        </p>
                      )}
                    </div>
                  )}
                />
              </>
            )}
          </Stepper.Body>
        </Stepper.Item>
        <Stepper.Item>
          <Stepper.Header>
            <Stepper.Icon>
              <HardDrive className="h-5 w-5" />
            </Stepper.Icon>
            <Stepper.Title>Ressources</Stepper.Title>
          </Stepper.Header>
          <Stepper.Body>
            {() => (
              <>
                <div className="text-sm">
                  Define limits & request for the service
                </div>
                <div className="flex flex-col gap-1">
                  <p className="font-medium text-sm">Request</p>
                  <p className="text-sm italic">
                    This number represents what Brume needs to allocated in
                    minimum for the service
                  </p>
                  <div className="flex flex-row gap-3">
                    <div>
                      <Label className="text-xs" htmlFor="request-cpu">
                        CPU
                      </Label>
                      <FormField
                        control={form.control}
                        name="data.cpu.request"
                        render={({ field, fieldState }) => (
                          <div className="flex flex-col gap-2">
                            <Input
                              {...field}
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
                              id="request-cpu"
                              onChange={(e) => {
                                field.onChange(Number(e.target.value));
                              }}
                              placeholder="0.1CPU"
                              step={0.1}
                              type="number"
                            />
                            {fieldState.error && (
                              <p className="text-red-500 text-xs">
                                {fieldState.error.message}
                              </p>
                            )}
                            {runner &&
                              draftRunner &&
                              draftRunner.data.cpu.request !==
                                runner.data.cpu.request && (
                                <p className="text-blue-500 text-xs italic">
                                  old value: {runner.data.cpu.request}
                                </p>
                              )}
                          </div>
                        )}
                      />
                    </div>
                    <div>
                      <Label className="text-xs" htmlFor="request-cpu">
                        Memory
                      </Label>
                      <FormField
                        control={form.control}
                        name="data.memory.request"
                        render={({ field, fieldState }) => (
                          <div className="flex flex-col gap-2">
                            <Input
                              {...field}
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
                              id="request-cpu"
                              onChange={(e) => {
                                field.onChange(Number(e.target.value));
                              }}
                              placeholder="100Mb"
                              step={100}
                              type="number"
                            />
                            {fieldState.error && (
                              <p className="text-red-500 text-xs">
                                {fieldState.error.message}
                              </p>
                            )}
                            {runner &&
                              draftRunner &&
                              draftRunner.data.memory.request !==
                                runner.data.memory.request && (
                                <p className="text-blue-500 text-xs italic">
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
                  <p className="font-medium text-sm">Limits</p>
                  <p className="text-sm italic">
                    This number represents what Brume needs to allocated in{" "}
                    <span className="font-medium">maximum</span> for this
                    service. If the value is exceded, the service will be
                    stopped
                  </p>
                  <div className="flex flex-row gap-3">
                    <div>
                      <Label className="text-xs" htmlFor="request-cpu">
                        CPU
                      </Label>
                      <FormField
                        control={form.control}
                        name="data.cpu.limit"
                        render={({ field, fieldState }) => (
                          <div className="flex flex-col gap-2">
                            <Input
                              {...field}
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
                              id="request-cpu"
                              onChange={(e) => {
                                field.onChange(Number(e.target.value));
                              }}
                              placeholder="0.2CPU"
                              step={0.1}
                              type="number"
                            />
                            {fieldState.error && (
                              <p className="text-red-500 text-xs">
                                {fieldState.error.message}
                              </p>
                            )}
                            {runner &&
                              draftRunner &&
                              draftRunner.data.cpu.limit !==
                                runner.data.cpu.limit && (
                                <p className="text-blue-500 text-xs italic">
                                  old value: {runner.data.cpu.limit}
                                </p>
                              )}
                          </div>
                        )}
                      />
                    </div>
                    <div>
                      <Label className="text-xs" htmlFor="request-cpu">
                        Memory
                      </Label>
                      <FormField
                        control={form.control}
                        name="data.memory.limit"
                        render={({ field, fieldState }) => (
                          <div className="flex flex-col gap-2">
                            <Input
                              {...field}
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
                              id="request-cpu"
                              onChange={(e) => {
                                field.onChange(Number(e.target.value));
                              }}
                              placeholder="150Mb"
                              step={100}
                              type="number"
                            />
                            {fieldState.error && (
                              <p className="text-red-500 text-xs">
                                {fieldState.error.message}
                              </p>
                            )}
                            {runner &&
                              draftRunner &&
                              draftRunner.data.memory.limit !==
                                runner.data.memory.limit && (
                                <p className="text-blue-500 text-xs italic">
                                  old value: {runner.data.memory.limit}
                                </p>
                              )}
                          </div>
                        )}
                      />
                    </div>
                  </div>
                </div>
              </>
            )}
          </Stepper.Body>
        </Stepper.Item>
        <Stepper.Item className="h-full pb-32">
          <Stepper.Header>
            <Stepper.Icon>
              <Network className="h-5 w-5" />
            </Stepper.Icon>
            <Stepper.Title>Network</Stepper.Title>
          </Stepper.Header>
          <Stepper.Body>
            {() => (
              <div className="flex flex-col gap-2">
                <div className="flex flex-col gap-1">
                  <p className="font-medium text-sm">Public networking</p>
                  <p className="text-sm">
                    Access your application over HTTPS with the following
                    domains.
                  </p>
                </div>
                <div className="flex flex-col gap-1">
                  <Label className="text-xs" htmlFor="domain">
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
                            id="domain"
                            placeholder="my-app.brume.run"
                            type="text"
                          />
                          <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                            <span className="text-muted-foreground text-sm">
                              .brume.run
                            </span>
                          </div>
                        </div>
                        {fieldState.error && (
                          <p className="text-red-500 text-xs">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.publicDomain !==
                            runner.data.publicDomain && (
                            <p className="text-blue-500 text-xs italic">
                              old value: {runner.data.publicDomain}
                            </p>
                          )}
                      </div>
                    )}
                  />
                </div>
                <div className="flex flex-col gap-1 pt-4">
                  <div className="flex flex-col gap-1">
                    <p className="font-medium text-sm">Private networking</p>
                    <p className="text-sm">
                      Communicate with other services inside Brume private
                      network.
                    </p>
                  </div>
                  <div className="flex flex-col gap-1">
                    <Label className="text-xs" htmlFor="domain">
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
                              placeholder="chat"
                              type="text"
                            />
                            <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                              <span className="text-muted-foreground text-sm">
                                .brume.internal
                              </span>
                            </div>
                          </div>
                          {fieldState.error && (
                            <p className="text-red-500 text-xs">
                              {fieldState.error.message}
                            </p>
                          )}
                          {runner &&
                            draftRunner &&
                            draftRunner.data.privateDomain !==
                              runner.data.privateDomain && (
                              <p className="text-blue-500 text-xs italic">
                                old value: {runner.data.privateDomain}
                              </p>
                            )}
                        </div>
                      )}
                    />
                  </div>
                </div>
              </div>
            )}
          </Stepper.Body>
        </Stepper.Item>
      </Stepper.Root>
    </Form>
  );
};

export const OldRunnerPage = () => {
  const { serviceId } = useParams<RouteParams>();
  const [wasDraft, setWasDraft] = useState(false);

  const { data: service, complete } = useFragment({
    from: `Service:${serviceId}`,
    fragment: ServiceFragment,
    fragmentName: "ServiceFragment",
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
      if (draftRunner) {
        return draftRunner;
      }
      if (runner) {
        return runner;
      }
      return;
    }, [draftRunner, runner]),
  });

  const blocker = useBlocker(() => form.formState.isDirty);

  if (blocker.state === "blocked") {
    toast.warning("You have unsaved changes");
  }

  useEffect(() => {
    if (service) {
      console.log(service);
      if (!(draftRunner || runner)) {
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
  }, [form, runner, service, draftRunner]);

  useEffect(() => {
    if (draftRunner) {
      setWasDraft(true);
    }
    if (wasDraft && !draftRunner && runner) {
      form.reset(runner);
      setWasDraft(false);
    }
  }, [runner, draftRunner, wasDraft, form.reset]);

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
    if (!service?.id) {
      return;
    }

    await updateRunnerMutation({
      variables: {
        serviceId: service.id,
        input: form.getValues().data,
      },
    });

    toast.success("Runner updated");
    form.reset(form.getValues());
  };

  if (!service) {
    return null;
  }

  return (
    <Form {...form}>
      <div className="flex h-full flex-col px-32 pt-8">
        <div className="relative flex flex-row items-center justify-between">
          <div className="absolute top-[-150px] right-0">
            {form.formState.isDirty && (
              <div className="flex flex-row items-center space-x-2">
                {loading && <Loader2 className="h-4 w-4 animate-spin" />}
                <Button
                  className="text-xs"
                  disabled={Object.keys(form.formState.errors).length > 0}
                  onClick={submitChanges}
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
        <div className="flex h-full flex-col pt-3">
          <div className="relative flex max-w-[700px] flex-col space-y-4 border-gray-300 border-l pb-16 pl-4">
            <div className="flex flex-row items-center">
              <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
                <SquareTerminal className="h-5 w-5" />
              </div>
              <div className="pl-4">Command</div>
            </div>
            <p className="font-medium text-sm">Custom start command</p>
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
                    className={cn(
                      "w-[300px] font-mono",
                      runner &&
                        draftRunner &&
                        draftRunner.data.command !== runner.data.command &&
                        "border-blue-500",
                      fieldState.isDirty && "border-green-500"
                    )}
                    placeholder="npx run start"
                  />
                  {runner &&
                    draftRunner &&
                    draftRunner.data.command !== runner.data.command && (
                      <p className="text-blue-500 text-xs italic">
                        old value: {runner.data.command}
                      </p>
                    )}
                </div>
              )}
            />
          </div>
          <div className="relative flex max-w-[700px] flex-col space-y-4 border-gray-300 border-l pb-16 pl-4">
            <div className="flex flex-row items-center">
              <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
                <Activity className="h-5 w-5" />
              </div>
              <div className="pl-4">Status</div>
            </div>
            <Alert className="border-green-400 bg-green-50" variant="default">
              <RocketIcon className="h-4 w-4" color="green" />
              <AlertTitle className="text-green-700">
                Service Healthy
              </AlertTitle>
              <AlertDescription className="text-green-800">
                Last health check showed that the service is healthy
              </AlertDescription>
            </Alert>
            <p className="font-medium text-sm">Healthcheck</p>
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
                    placeholder="http://localhost:8080/healthz"
                  />
                  {runner &&
                    draftRunner &&
                    draftRunner.data.healthCheckURL !==
                      runner.data.healthCheckURL && (
                      <p className="text-blue-500 text-xs italic">
                        old value: {runner.data.healthCheckURL}
                      </p>
                    )}
                  {fieldState.error && (
                    <p className="text-red-500 text-xs">
                      {fieldState.error.message}
                    </p>
                  )}
                </div>
              )}
            />
          </div>
          <div className="relative flex flex-col space-y-4 border-gray-300 border-l pb-16 pl-4">
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
              <p className="font-medium text-sm">Request</p>
              <p className="text-sm italic">
                This number represents what Brume needs to allocated in minimum
                for the service
              </p>
              <div className="flex flex-row gap-3">
                <div>
                  <Label className="text-xs" htmlFor="request-cpu">
                    CPU
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.cpu.request"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
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
                          id="request-cpu"
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          placeholder="0.1CPU"
                          step={0.1}
                          type="number"
                        />
                        {fieldState.error && (
                          <p className="text-red-500 text-xs">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.cpu.request !==
                            runner.data.cpu.request && (
                            <p className="text-blue-500 text-xs italic">
                              old value: {runner.data.cpu.request}
                            </p>
                          )}
                      </div>
                    )}
                  />
                </div>
                <div>
                  <Label className="text-xs" htmlFor="request-cpu">
                    Memory
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.memory.request"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
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
                          id="request-cpu"
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          placeholder="100Mb"
                          step={100}
                          type="number"
                        />
                        {fieldState.error && (
                          <p className="text-red-500 text-xs">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.memory.request !==
                            runner.data.memory.request && (
                            <p className="text-blue-500 text-xs italic">
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
              <p className="font-medium text-sm">Limits</p>
              <p className="text-sm italic">
                This number represents what Brume needs to allocated in{" "}
                <span className="font-medium">maximum</span> for this service.
                If the value is exceded, the service will be stopped
              </p>
              <div className="flex flex-row gap-3">
                <div>
                  <Label className="text-xs" htmlFor="request-cpu">
                    CPU
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.cpu.limit"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
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
                          id="request-cpu"
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          placeholder="0.2CPU"
                          step={0.1}
                          type="number"
                        />
                        {fieldState.error && (
                          <p className="text-red-500 text-xs">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.cpu.limit !==
                            runner.data.cpu.limit && (
                            <p className="text-blue-500 text-xs italic">
                              old value: {runner.data.cpu.limit}
                            </p>
                          )}
                      </div>
                    )}
                  />
                </div>
                <div>
                  <Label className="text-xs" htmlFor="request-cpu">
                    Memory
                  </Label>
                  <FormField
                    control={form.control}
                    name="data.memory.limit"
                    render={({ field, fieldState }) => (
                      <div className="flex flex-col gap-2">
                        <Input
                          {...field}
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
                          id="request-cpu"
                          onChange={(e) => {
                            field.onChange(Number(e.target.value));
                          }}
                          placeholder="150Mb"
                          step={100}
                          type="number"
                        />
                        {fieldState.error && (
                          <p className="text-red-500 text-xs">
                            {fieldState.error.message}
                          </p>
                        )}
                        {runner &&
                          draftRunner &&
                          draftRunner.data.memory.limit !==
                            runner.data.memory.limit && (
                            <p className="text-blue-500 text-xs italic">
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
          <div className="relative flex flex-col space-y-4 border-gray-300 border-l pb-16 pl-4">
            <div className="flex flex-row items-center">
              <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
                <Network className="h-5 w-5" />
              </div>
              <div className="pl-4">Network</div>
            </div>
            <div className="flex flex-col gap-3">
              <div className="flex flex-col gap-1">
                <p className="font-medium text-sm">Public networking</p>
                <p className="text-sm">
                  Access your application over HTTPS with the following domains.
                </p>
              </div>
              <div className="flex flex-col gap-1">
                <Label className="text-xs" htmlFor="domain">
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
                          id="domain"
                          placeholder="my-app.brume.run"
                          type="text"
                        />
                        <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                          <span className="text-muted-foreground text-sm">
                            .brume.run
                          </span>
                        </div>
                      </div>
                      {fieldState.error && (
                        <p className="text-red-500 text-xs">
                          {fieldState.error.message}
                        </p>
                      )}
                      {runner &&
                        draftRunner &&
                        draftRunner.data.publicDomain !==
                          runner.data.publicDomain && (
                          <p className="text-blue-500 text-xs italic">
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
                <p className="font-medium text-sm">Private networking</p>
                <p className="text-sm">
                  Communicate with other services inside Brume private network.
                </p>
              </div>
              <div className="flex flex-col gap-1">
                <Label className="text-xs" htmlFor="domain">
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
                          placeholder="chat"
                          type="text"
                        />
                        <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                          <span className="text-muted-foreground text-sm">
                            .brume.internal
                          </span>
                        </div>
                      </div>
                      {fieldState.error && (
                        <p className="text-red-500 text-xs">
                          {fieldState.error.message}
                        </p>
                      )}
                      {runner &&
                        draftRunner &&
                        draftRunner.data.privateDomain !==
                          runner.data.privateDomain && (
                          <p className="text-blue-500 text-xs italic">
                            old value: {runner.data.privateDomain}
                          </p>
                        )}
                    </div>
                  )}
                />
              </div>
            </div>
          </div>
          <div className="grow border-gray-300 border-l" />
        </div>
      </div>
    </Form>
  );
};
