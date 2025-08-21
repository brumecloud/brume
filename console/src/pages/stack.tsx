import { MachineCard } from "@/components/cards/machine";
import { Page } from "@/components/page-comp/header";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
    <Page.Container>
      <Page.Header>
        <Page.Title>Stack</Page.Title>
        <Page.Description>
          See all the differents stacks you have created
        </Page.Description>
      </Page.Header>
      <Page.Body className="flex flex-col gap-8">
        <div className="flex flex-row gap-4">
          <Input placeholder="Search all your stacks" />
          <NavLink to="/overview/marketplace">
            <Button>Add new...</Button>
          </NavLink>
        </div>
        <div className="grid grid-cols-6 gap-4">
          {/*<div className="col-span-2 flex flex-col gap-2">
            <h3>Recent deployments</h3>
            <div className="min-h-32 w-full rounded-md border"></div>
          </div>*/}
          <div className="col-span-6 flex flex-col gap-2">
            {/*<h3>Stacks</h3>*/}
            <div className="grid grid-cols-3">
              {machines?.map((machine) => (
                <div
                  key={machine.id}
                  className="flex w-full flex-col gap-2 rounded-md border p-2">
                  <h4>{machine.name}</h4>
                  <p>{machine.ip}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </Page.Body>
    </Page.Container>
  );
  //   <div className="flex flex-col">
  //     <div className="px-32 pt-8">
  //       <div className="flex flex-row items-center justify-between pt-16">
  //         <div className="flex flex-col pb-8">
  //           <h2 className="pb-2 font-heading text-3xl">Stack</h2>
  //           <p>Manage and operate your differents stacks</p>
  //         </div>
  //         <NavLink to="/stack/marketplace">
  //           <Button>Add a new stack</Button>
  //         </NavLink>
  //       </div>
  //       <div className="flex flex-row gap-4">
  //         {machines?.map((machine) => (
  //           <MachineCard id={machine.id} />
  //         ))}
  //       </div>
  //     </div>
  //   </div>
  // );
}
