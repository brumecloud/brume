import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { GoCheckCircleFill } from "react-icons/go";
import { IoFolderOpenOutline } from "react-icons/io5";
import { MdDriveFileRenameOutline } from "react-icons/md";
import { MdDns } from "react-icons/md";

export const AddDomain = () => {
  return (
    <Page.Container>
      <Page.Header>
        <Page.Title>Add an existing domain</Page.Title>
        <Page.Description>
          Link a domain that you already own to your account using DNS
          NS records.
        </Page.Description>
      </Page.Header>
      <Page.Body className="pt-8">
        <Stepper.Root shouldAnimate leftBorder>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <MdDriveFileRenameOutline />
              </Stepper.Icon>
              <Stepper.Title>Choose domain name</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="text-sm font-medium">
                    Enter the domain name
                  </div>
                  <p className="text-sm text-gray-500">
                    Choose a domain name you own and you want access
                    to the DNS records
                  </p>
                  <Input
                    placeholder="brume.dev"
                    className="max-w-96"
                  />
                  <Button className="w-32" onClick={() => setStep(1)}>
                    Validate
                  </Button>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <MdDns />
              </Stepper.Icon>
              <Stepper.Title>Create NS record</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="text-sm font-medium">
                    Redirect your domain to our DNS servers
                  </div>
                  <p className="text-sm text-gray-500">
                    The DNS records at your provider must match the
                    following records to verify and connect your
                    domain to Brume
                  </p>
                  <div className="w-64 rounded-md border bg-gray-100 p-4 font-mono text-sm">
                    <p>NS ns1.brume.dev</p>
                    <p>NS ns2.brume.dev</p>
                  </div>
                  <Button className="w-32" onClick={() => setStep(2)}>
                    Copy
                  </Button>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <GoCheckCircleFill />
              </Stepper.Icon>
              <Stepper.Title>Verify connection</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="text-sm font-medium">
                    We are checking everything
                  </div>
                  <p className="text-sm text-gray-500">
                    DNS records can take time to propagate. This can
                    take a few minutes
                  </p>
                  <div className="flex flex-row gap-2">
                    <Button variant={"outline"}>Refresh</Button>
                    <Button
                      onClick={() => {
                        setStep(3);
                      }}>
                      Validate
                    </Button>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item className="h-full">
            <Stepper.Header>
              <Stepper.Icon>
                <IoFolderOpenOutline />
              </Stepper.Icon>
              <Stepper.Title>Link to a project</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="text-sm font-medium">
                    Everything looks good!
                  </div>
                  <p className="text-sm text-gray-500">
                    You can now link your new domain to one of your
                    projects.
                  </p>
                  <div>
                    <Select>
                      <SelectTrigger className="w-[300px]">
                        <SelectValue placeholder="Select a project" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="porfolio">
                          Porfolio
                        </SelectItem>
                        <SelectItem value="chat-app">
                          vibe-coded-chat
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex flex-row gap-2">
                    <Button variant={"outline"}>Skip</Button>
                    <Button
                      onClick={() => {
                        setStep(4);
                      }}>
                      Link to project
                    </Button>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
        </Stepper.Root>
      </Page.Body>
    </Page.Container>
  );
};
