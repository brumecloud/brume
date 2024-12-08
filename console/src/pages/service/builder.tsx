import { Button } from "@/components/ui/button";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useService } from "@/hooks/useService";
import { useUpdateBuilder } from "@/hooks/useUpdateBuilder";
import {
  BuilderSchema,
  type Builder,
} from "@/schemas/service.schema";
import { cn } from "@/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  ArrowUpFromLine,
  Code,
  Loader2,
  Pickaxe,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Form, useForm } from "react-hook-form";
import { useBlocker } from "react-router-dom";
import { toast } from "sonner";
import { Label } from "src/components/ui/label";
import {
  RadioGroup,
  RadioGroupItem,
} from "src/components/ui/radio-group";

export const BuildTypeValues = {
  GenericImage: "generic-image",
  Dockerfile: "dockerfile",
  StaticWebsite: "static-website",
} as const;

export type BuildType =
  (typeof BuildTypeValues)[keyof typeof BuildTypeValues];

const GenericImageOptions = () => {
  const { service } = useService();
  const [wasDraft, setWasDraft] = useState(false);
  const draftBuilder = service?.draftBuilder;
  const builder = service?.liveBuilder;

  const { updateBuilderMutation, loading } = useUpdateBuilder();

  const form = useForm<Builder>({
    resolver: zodResolver(BuilderSchema),
    mode: "onChange",
    defaultValues: useMemo(() => {
      if (draftBuilder) {
        return draftBuilder;
      } else if (builder) {
        return builder;
      }
      return undefined;
    }, [draftBuilder, builder]),
  });

  const blocker = useBlocker(() => {
    return form.formState.isDirty;
  });

  if (blocker.state === "blocked" && form.formState.isDirty) {
    toast.warning("You have unsaved changes");
  }

  useEffect(() => {
    if (service) {
      if (draftBuilder) {
        form.reset(draftBuilder);
      } else if (builder) {
        form.reset(builder);
      }
    }
  }, [form, service?.id]);

  useEffect(() => {
    if (draftBuilder) {
      setWasDraft(true);
    }
    if (wasDraft && !draftBuilder && builder) {
      form.reset(builder);
      setWasDraft(false);
    }
  }, [service]);

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

    await updateBuilderMutation({
      variables: {
        serviceId: service.id,
        input: form.getValues().data,
      },
    });

    toast.success("Builder updated");
    form.reset(form.getValues());
  };

  if (!service) return null;

  return (
    <Form {...form}>
      <div className="relative flex flex-col space-y-4 border-l border-gray-300 pb-16 pl-4 transition-all duration-100">
        <div className="flex h-12 flex-row items-center">
          <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
            <Code className="h-5 w-5" />
          </div>
          <div className="flex w-full flex-row items-center justify-between space-x-2 pl-3">
            <div className="text-sm font-medium">Docker image</div>
            {form.formState.isDirty && (
              <div className="flex flex-row items-center space-x-2">
                {loading && (
                  <Loader2 className="h-4 w-4 animate-spin" />
                )}
                <Button
                  onClick={submitChanges}
                  variant="outline"
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
        <div className="flex flex-col space-y-1">
          <p className="text-sm font-medium">Image information</p>
          <p className="text-sm">
            Choose a registry and provide a valid docker image name.
          </p>
        </div>
        <div className="flex flex-row gap-2">
          <div className="flex flex-col space-y-2">
            <FormField
              control={form.control}
              name="data.registry"
              render={({ field }) => (
                <>
                  <Label htmlFor="registry" className="text-xs">
                    Registry
                  </Label>
                  <Select {...field}>
                    <SelectTrigger
                      className={cn(
                        "w-[180px]",
                        builder &&
                          draftBuilder &&
                          draftBuilder.data.registry !==
                            builder.data.registry &&
                          "border-blue-500",
                        form.formState.dirtyFields.data?.registry &&
                          "border-green-500"
                      )}>
                      <SelectValue placeholder="Registry" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="docker.io">
                        Docker Hub
                      </SelectItem>
                      <SelectItem value="ghcr.io">
                        GitHub Container Registry
                      </SelectItem>
                      <SelectItem value="quay.io">Quay</SelectItem>
                    </SelectContent>
                  </Select>
                  {builder &&
                    draftBuilder &&
                    draftBuilder.data.registry !==
                      builder.data.registry && (
                      <p className="text-xs italic text-blue-500">
                        old value: {builder.data.registry}
                      </p>
                    )}
                </>
              )}
            />
          </div>
          <div className="flex flex-col space-y-2">
            <FormField
              control={form.control}
              name="data.image"
              render={({ field }) => (
                <>
                  <Label htmlFor="image" className="text-xs">
                    Image
                  </Label>
                  <Input
                    {...field}
                    id="image"
                    type="text"
                    placeholder="hello-world"
                    className={cn(
                      builder &&
                        draftBuilder &&
                        draftBuilder.data.image !==
                          builder.data.image &&
                        "border-blue-500",
                      form.formState.dirtyFields.data?.image &&
                        "border-green-500"
                    )}
                  />
                  {builder &&
                    draftBuilder &&
                    draftBuilder.data.image !==
                      builder.data.image && (
                      <p className="text-xs italic text-blue-500">
                        old value: {builder.data.image}
                      </p>
                    )}
                </>
              )}
            />
          </div>
          <div className="flex flex-col space-y-2">
            <FormField
              control={form.control}
              name="data.tag"
              render={({ field }) => (
                <>
                  <Label htmlFor="tag" className="text-xs">
                    Tag
                  </Label>
                  <Input
                    {...field}
                    id="tag"
                    type="text"
                    placeholder="latest"
                    className={cn(
                      builder &&
                        draftBuilder &&
                        draftBuilder.data.tag !== builder.data.tag &&
                        "border-blue-500",
                      form.formState.dirtyFields.data?.tag &&
                        "border-green-500"
                    )}
                  />
                  {builder &&
                    draftBuilder &&
                    draftBuilder.data.tag !== builder.data.tag && (
                      <p className="text-xs italic text-blue-500">
                        old value: {builder.data.tag}
                      </p>
                    )}
                </>
              )}
            />
          </div>
        </div>
      </div>
    </Form>
  );
};

const DockerfileOptions = () => {
  return (
    <div className="flex h-full flex-col">
      <div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
        <div className="flex flex-row items-center">
          <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
            <Code className="h-5 w-5" />
          </div>
          <div className="pl-4">Source code</div>
        </div>
        <div className="pt-4">test</div>
      </div>
      <div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
        <div className="flex flex-row items-center">
          <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
            <Pickaxe className="h-5 w-5" />
          </div>
          <div className="pl-4">Build artifact</div>
        </div>
        <div className="pt-4">test</div>
      </div>
      <div className="relative flex flex-col border-l border-gray-300 pb-4 pl-4">
        <div className="flex flex-row items-center">
          <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
            <ArrowUpFromLine className="h-5 w-5" />
          </div>
          <div className="pl-4">Push Artifact</div>
        </div>
        <div className="pt-4">test</div>
      </div>
    </div>
  );
};

const StaticWebsiteOptions = () => {
  return (
    <div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
      <div className="flex flex-row items-center">
        <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
          <Code className="h-5 w-5" />
        </div>
        <div className="pl-4">Static website</div>
      </div>
    </div>
  );
};

export const BuilderPage = () => {
  const [buildType, setBuildType] =
    useState<BuildType>("generic-image");

  return (
    <div className="flex h-full flex-col px-32 pt-8">
      <div className="py-16">
        <h2 className="font-heading pb-2 text-3xl">Builder</h2>
        <p>Configure the building pipeline workflow</p>
      </div>

      <div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
        <div className="flex flex-row items-center">
          <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
            <Pickaxe className="h-5 w-5" />
          </div>
          <div className="pl-4">Builder configuration</div>
        </div>
        <div className="flex flex-col space-y-4 pt-4">
          <div className="flex flex-col space-y-1">
            <div className="text-sm font-medium">Type of build</div>
            <p className="text-sm text-gray-500">
              The type of build to use for this service.
            </p>
          </div>
          <RadioGroup
            defaultValue="generic-image"
            onValueChange={(value) =>
              setBuildType(value as BuildType)
            }
            className="flex flex-col space-y-1">
            <div className="flex space-x-2">
              <RadioGroupItem
                value="generic-image"
                id="generic-image"
              />
              <Label
                htmlFor="generic-image"
                className="flex flex-col space-y-1">
                <span className="font-medium">Generic Image</span>
                <p className="text-sm text-gray-500">
                  A simple docker image available from a registered
                  docker registry.
                </p>
              </Label>
            </div>
            <div className="flex space-x-2">
              <RadioGroupItem
                value="dockerfile"
                id="dockerfile"
                disabled
              />
              <Label
                htmlFor="dockerfile"
                className="flex flex-col space-y-1">
                <span className="font-medium">Dockerfile</span>
                <p className="text-sm text-gray-500">
                  The artifact is a image, if a Dockerfile is present
                  in the artifact it will be used to build the image.
                  Otherwise, an automatic dockerfile will be created
                  using Nixpack
                </p>
              </Label>
            </div>
            <div className="flex space-x-2">
              <RadioGroupItem
                value="static-website"
                id="static-website"
                disabled
              />
              <Label
                htmlFor="static-website"
                className="flex flex-col space-y-1">
                <span className="font-medium">Static Website</span>
                <p className="text-sm text-gray-500">
                  The artifact is a static website. The artifact will
                  be served using a webserver (or a CDN like)
                </p>
              </Label>
            </div>
          </RadioGroup>
        </div>
      </div>
      {buildType === "generic-image" && <GenericImageOptions />}
      {buildType === "dockerfile" && <DockerfileOptions />}
      {buildType === "static-website" && <StaticWebsiteOptions />}
      <div className="grow border-l border-gray-300" />
    </div>
  );
};
