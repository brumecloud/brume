import { MachineCard } from "@/components/cards/machine";
import { GET_MACHINES } from "@/gql/machine.graphql";
import { useQuery } from "@apollo/client";

export default function Machines() {
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
            <h2 className="pb-2 font-heading text-3xl">Machines</h2>
            <p>
              Manage your army of machine running your applications
            </p>
          </div>
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
