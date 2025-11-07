import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { FaRegCopy } from "react-icons/fa6";

export const AccountPage = () => (
  <div className="flex flex-col items-start gap-8 pt-8">
    <div className="flex w-3/4 flex-col gap-2 rounded-xl border">
      <div className="flex flex-col gap-4 p-8 pb-4">
        <h2 className="font-semibold text-lg">Account Name</h2>
        <p>
          This is your account visible name on Brume. Be careful while changing
          it
        </p>
        <Input value={"Brume Admin"} className="max-w-96" />
      </div>
      <div className="flex flex-row items-center justify-end border-0 border-t p-4 py-2">
        <Button>save</Button>
      </div>
    </div>
    <div className="flex w-3/4 flex-col gap-2 rounded-xl border">
      <div className="flex flex-col gap-4 p-8 pb-4">
        <h2 className="font-semibold text-lg">Account ID</h2>
        <p>This is your Brume Account ID.</p>
        <div className="flex w-fit flex-row items-center gap-2 rounded-sm border p-3 font-mono text-gray-700 text-sm">
          1234567890abcdef1234567890abcdef
          <FaRegCopy className="cursor-pointer text-gray-400 hover:text-gray-600" />
        </div>
      </div>
      <div className="flex flex-row items-center justify-start border-0 border-t px-8 py-4 text-gray-400 text-sm">
        You can use it with CLI or API.
      </div>
    </div>
  </div>
);
