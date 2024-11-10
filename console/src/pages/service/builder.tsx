import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ArrowUpFromLine, Code, Pickaxe } from "lucide-react";
import { useState } from "react";
import { Label } from "src/components/ui/label";
import { RadioGroup, RadioGroupItem } from "src/components/ui/radio-group";

export const BuildTypeValues = {
  GenericImage: "generic-image",
  Dockerfile: "dockerfile",
  StaticWebsite: "static-website",
} as const;

export type BuildType = (typeof BuildTypeValues)[keyof typeof BuildTypeValues];

const GenericImageOptions = () => {
  return (
    <div className="relative flex flex-col space-y-4 border-l border-gray-300 pb-16 pl-4">
      <div className="flex flex-row items-center">
        <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
          <Code className="h-5 w-5" />
        </div>
        <div className="pl-4">Docker image</div>
      </div>
      <div className="flex flex-col space-y-1">
        <p className="text-sm font-medium">Image information</p>
        <p className="text-sm">Choose a registry and provide a valid docker image name.</p>
      </div>
      <div className="flex flex-row gap-2">
        <div className="flex flex-col space-y-2">
          <Label htmlFor="registry" className="text-xs">
            Registry
          </Label>
          <Select>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Registry" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="docker.io">Docker Hub</SelectItem>
              <SelectItem value="ghcr.io">GitHub Container Registry</SelectItem>
              <SelectItem value="quay.io">Quay</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col space-y-2">
          <Label htmlFor="image" className="text-xs">
            Image
          </Label>
          <Input id="image" type="text" placeholder="nginx" />
        </div>
        <div className="flex flex-col space-y-2">
          <Label htmlFor="tag" className="text-xs">
            Tag
          </Label>
          <Select>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="latest" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="latest">latest</SelectItem>
              <SelectItem value="v1.0.0">v1.0.0</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
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
  const [buildType, setBuildType] = useState<BuildType>("generic-image");

  return (
    <div className="flex h-full flex-col px-32 pt-8">
      <div className="py-16">
        <h2 className="pb-2 text-2xl font-semibold">Builder</h2>
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
            <p className="text-sm text-gray-500">The type of build to use for this service.</p>
          </div>
          <RadioGroup
            defaultValue="generic-image"
            onValueChange={(value) => setBuildType(value as BuildType)}
            className="flex flex-col space-y-1">
            <div className="flex space-x-2">
              <RadioGroupItem value="generic-image" id="generic-image" />
              <Label htmlFor="generic-image" className="flex flex-col space-y-1">
                <span className="font-medium">Generic Image</span>
                <p className="text-sm text-gray-500">
                  A simple docker image available from a registered docker registry.
                </p>
              </Label>
            </div>
            <div className="flex space-x-2">
              <RadioGroupItem value="dockerfile" id="dockerfile" disabled />
              <Label htmlFor="dockerfile" className="flex flex-col space-y-1">
                <span className="font-medium">Dockerfile</span>
                <p className="text-sm text-gray-500">
                  The artifact is a image, if a Dockerfile is present in the artifact it will be used to build
                  the image. Otherwise, an automatic dockerfile will be created using Nixpack
                </p>
              </Label>
            </div>
            <div className="flex space-x-2">
              <RadioGroupItem value="static-website" id="static-website" disabled />
              <Label htmlFor="static-website" className="flex flex-col space-y-1">
                <span className="font-medium">Static Website</span>
                <p className="text-sm text-gray-500">
                  The artifact is a static website. The artifact will be served using a webserver (or a CDN
                  like)
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
