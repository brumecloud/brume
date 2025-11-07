import { useQuery } from "@apollo/client";
import ArchitectureServiceAmazonCloudFront from "aws-react-icons/lib/icons/ArchitectureServiceAmazonCloudFront";
import ArchitectureServiceAWSPrivateCertificateAuthority from "aws-react-icons/lib/icons/ArchitectureServiceAWSPrivateCertificateAuthority";
import ResourceAmazonSimpleStorageServiceBucket from "aws-react-icons/lib/icons/ResourceAmazonSimpleStorageServiceBucket";
import { useState } from "react";
import { BiPulse } from "react-icons/bi";
import { NavLink } from "react-router-dom";
import { Page } from "@/components/page-comp/header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { GET_MACHINES } from "@/gql/machine.graphql";

const StackCard = () => {
  const [isHovering, setIsHovering] = useState(false);
  return (
    <div className="min-w-[400px] overflow-hidden rounded-md border">
      <div className="flex h-16 flex-row items-center justify-between border-b px-3">
        <div className="flex flex-col gap-[3px]">
          <h2>Frontend stack</h2>
          <div className="flex flex-row gap-2">
            <Badge>Production account</Badge>
            <Badge variant={"outline"}>AWS</Badge>
          </div>
        </div>
        <div className="flex flex-row items-center justify-center gap-x-3 pr-2">
          <div className="rounded-full border border-green-300 bg-green-50 p-1">
            <BiPulse className="size-4 text-green-500" />
          </div>
        </div>
      </div>
      {/** biome-ignore lint/a11y/noStaticElementInteractions: need */}
      <div
        className="-z-10 inset-0 flex h-40 w-full items-center justify-center gap-x-2 bg-[radial-gradient(circle,#73737350_1px,transparent_1px)] bg-[size:10px_10px] bg-gray-50"
        onMouseEnter={() => setIsHovering(true)}
        onMouseLeave={() => setIsHovering(false)}
      >
        {isHovering ? (
          <div>
            <NavLink to="/overview/stack/id">
              <Button>View stack</Button>
            </NavLink>
          </div>
        ) : (
          <div className="flex items-center justify-center gap-x-2">
            <ResourceAmazonSimpleStorageServiceBucket className="size-10 bg-gray-50" />
            <ArchitectureServiceAmazonCloudFront className="size-10 rounded-sm bg-gray-50" />
            <ArchitectureServiceAWSPrivateCertificateAuthority className="size-10 rounded-sm bg-gray-50" />
          </div>
        )}
      </div>
    </div>
  );
};

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
              {machines?.map((_machine) => (
                <StackCard />
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
