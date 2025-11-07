import { Button } from "@/components/ui/button";
import { TbWorldSearch } from "react-icons/tb";
import { NavLink } from "react-router-dom";

export const DomainPage = () => {
  return (
    <div className="flex h-full pt-8">
      <div className="flex min-h-32 w-full flex-col gap-8 rounded-xl border p-2 pb-16">
        {/* header */}
        <div className="flex flex-row items-center justify-center gap-8 pt-8">
          <div className="flex flex-col items-center gap-2 pt-8">
            <div className="flex size-12 flex-col items-center justify-center rounded border">
              <TbWorldSearch className="size-8" />
            </div>
            <h2 className="font-medium text-lg">Add a domain</h2>
            <p className="text-gray-500 text-sm">
              Add a domain that you can connect to your projects.
            </p>
          </div>
        </div>
        <div className="m-auto gap-2 pt-4">
          <NavLink to="/settings/domains/add">
            <Button>Add existing domain</Button>
          </NavLink>
        </div>
      </div>
    </div>
  );
};
