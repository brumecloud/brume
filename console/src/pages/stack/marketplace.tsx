import { Page } from "@/components/page-comp/header";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import ArchitectureServiceAWSPrivateCertificateAuthority from "aws-react-icons/icons/ArchitectureServiceAWSPrivateCertificateAuthority";
import ArchitectureServiceAmazonCloudFront from "aws-react-icons/icons/ArchitectureServiceAmazonCloudFront";
import ResourceAmazonSimpleStorageServiceBucket from "aws-react-icons/icons/ResourceAmazonSimpleStorageServiceBucket";
import { useState } from "react";
import { FaAws } from "react-icons/fa";
import { LuBadgeCheck } from "react-icons/lu";
import { NavLink } from "react-router-dom";

const StackCard = () => {
  const [isHovering, setIsHovering] = useState(false);
  return (
    <div className="min-w-[400px] overflow-hidden rounded-md border">
      <div className="flex h-12 flex-row items-center justify-between border-b px-3">
        <h2>Single Page Application</h2>
        <div className="flex flex-row items-center justify-center gap-x-3">
          <FaAws className="size-6" />
          <LuBadgeCheck className="size-5" />
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
            <NavLink to="/overview/deploy?stack=spa-aws-brume">
              <Button>Deploy this stack</Button>
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

export const Marketplace = () => (
  <Page.Container>
    <Page.Header>
      <Page.Title>Marketplace</Page.Title>
      <Page.Description>
        Browse all the stack Brume has to offer. Every stack you see in this
        marketplace has been peer reviewed and is ready to deploy.
      </Page.Description>
    </Page.Header>
    <Page.Body className="flex flex-col gap-8">
      <div className="w-full">
        <Input placeholder="Search for a stack" />
      </div>
      <div className="flex flex-row">
        <StackCard />
      </div>
    </Page.Body>
  </Page.Container>
);
