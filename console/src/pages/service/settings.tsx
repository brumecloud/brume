import { Button } from "@/components/ui/button";
import { Bell, Flame } from "lucide-react";

export const SettingPage = () => {
  return (
    <div className="flex h-full flex-col px-32 pt-8">
      <div className="flex flex-col py-16">
        <h2 className="h-full pb-2 text-2xl font-semibold">
          Settings
        </h2>
        <p>Manage the service</p>
      </div>
      <div className="flex h-full flex-col">
        <div className="relative flex flex-col border-l border-gray-300 pb-16 pl-4">
          <div className="flex flex-row items-center">
            <div className="absolute left-[-20px] rounded-full border border-gray-300 bg-white p-2 text-gray-600">
              <Bell className="h-5 w-5" />
            </div>
            <div className="pl-4">Notifications</div>
          </div>
          <div className="pt-4">test</div>
        </div>
        <div className="relative flex flex-col border-l border-gray-300 pl-4">
          <div className="flex flex-row items-center">
            <div className="absolute left-[-20px] rounded-full border border-red-300 bg-white p-2 text-red-600">
              <Flame className="h-5 w-5" />
            </div>
            <div className="pl-4 text-red-800">Delete service</div>
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
