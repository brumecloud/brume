import {
  Alert,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Activity,
  HardDrive,
  Network,
  RocketIcon,
} from "lucide-react";
import { useState } from "react";

export const RunnerPage = () => {
  const [compute, setCompute] = useState("Brume.dev - Docker Runner");

  return (
    <div className="flex h-full flex-col px-32 pt-8">
      <div className="flex flex-col pb-8 pt-16">
        <h2 className="pb-2 text-2xl font-semibold">Runners</h2>
        <p>Inspect & configure how your code runs</p>
      </div>
      <div className="flex shrink-0 gap-6 border-b">
        <button
          data-state="active"
          className="border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800">
          Overview
        </button>
        <button className="border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800">
          Variable
        </button>
        <button className="border-gray-800 py-2 text-gray-500 data-[state=active]:border-b data-[state=active]:text-gray-800">
          Deployments
        </button>
      </div>
      <div className="flex h-full flex-col pt-8">
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
          <Input placeholder="http://localhost:8080/healthz" />
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
              This number represents what Brume needs to allocated in
              minimum for the service
            </p>
            <div className="flex flex-row gap-3">
              <div>
                <Label htmlFor="request-cpu" className="text-xs">
                  CPU
                </Label>
                <Input
                  id="request-cpu"
                  type="number"
                  placeholder="0.1CPU"
                />
              </div>
              <div>
                <Label htmlFor="request-cpu" className="text-xs">
                  Memory
                </Label>
                <Input
                  id="request-cpu"
                  type="number"
                  placeholder="100Mb"
                />
              </div>
            </div>
          </div>
          <div className="flex flex-col gap-1">
            <p className="text-sm font-medium">Limits</p>
            <p className="text-sm italic">
              This number represents what Brume needs to allocated in{" "}
              <span className="font-medium">maximum</span> for this
              service. If the value is exceded, the service will be
              stopped
            </p>
            <div className="flex flex-row gap-3">
              <div>
                <Label htmlFor="request-cpu" className="text-xs">
                  CPU
                </Label>
                <Input
                  id="request-cpu"
                  type="number"
                  placeholder="0.2CPU"
                />
              </div>
              <div>
                <Label htmlFor="request-cpu" className="text-xs">
                  Memory
                </Label>
                <Input
                  id="request-cpu"
                  type="number"
                  placeholder="150Mb"
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
              <p className="text-sm font-medium">Public networking</p>
              <p className="text-sm">Access your application over HTTPS with the following domains.</p>
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="domain" className="text-xs">
                Domain
              </Label>
              <Input id="domain" className="w-[300px]" type="text" value="my-app.brume.run" />
            </div>
          </div>
          <div className="flex flex-col gap-1">
            <div className="flex flex-col gap-1">
              <p className="text-sm font-medium">Private networking</p>
              <p className="text-sm">Communicate with other services inside Brume private network.</p>
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="domain" className="text-xs">
                Private domain
              </Label>
              <Input id="domain" className="w-[300px]" type="text" value="my-app.brume.internal" />
            </div>
          </div>
        </div>
        <div className="grow border-l border-gray-300" />
      </div>
    </div>
  );
};
