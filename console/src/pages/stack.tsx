import { MachineCard } from "@/components/cards/machine";
import { Button } from "@/components/ui/button";
import { GET_MACHINES } from "@/gql/machine.graphql";
import { useQuery } from "@apollo/client";
import { NavLink } from "react-router-dom";

export default function Stacks() {
  const { data, loading, error } = useQuery(GET_MACHINES, {
    fetchPolicy: "cache-first",
  });

  if (loading) {
    return <div>Loading...</div>;
  }

  if (error) {
    throw error;
  }

  const machines = data?.machine;

  return (
    <div className="flex flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row items-center justify-between pt-16">
          <div className="flex flex-col pb-8">
            <h2 className="pb-2 font-heading text-3xl">Stack</h2>
            <p>Manage and operate your differents stacks</p>
          </div>
          <NavLink to="/stack/marketplace">
            <Button>Add a new stack</Button>
          </NavLink>
        </div>
        <div className="flex flex-row gap-4">
          {machines?.map((machine) => (
            <MachineCard id={machine.id} />
          ))}
        </div>
      </div>
    </div>
  );
}
