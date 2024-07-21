import { ArrowUpFromLine, Code, Pickaxe } from "lucide-react";

export const BuilderPage = () => {
  return (
    <div className="flex h-full flex-col px-32 pt-8">
      <div className="py-16">
        <h2 className="pb-2 text-2xl font-semibold">Builder</h2>
        <p>Configure the building pipeline workflow</p>
      </div>
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
        <div className="grow border-l border-gray-300" />
      </div>
    </div>
  );
};
