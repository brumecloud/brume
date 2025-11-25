import { useMutation, useQuery } from "@apollo/client";
import { useState } from "react";
import { MdChecklist, MdPreview } from "react-icons/md";
import { TfiPackage } from "react-icons/tfi";
import { useNavigate, useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { GET_CLOUD_ACCOUNTS } from "@/gql/cloud.graphql";
import { DEPLOY_STACK } from "@/gql/stack.graphql";

export const DeployStack = () => {
  const { data, loading } = useQuery(GET_CLOUD_ACCOUNTS);
  // get from the query params
  const [searchParams] = useSearchParams();
  const [domain, setDomain] = useState("");
  const [name, setName] = useState("");
  const [agreement, setAgreement] = useState(false);
  const [_planReview, setPlanReview] = useState(false);
  const [applyStack, setApplyStack] = useState(false);
  const navigate = useNavigate();

  const templateId = searchParams.get("stack");
  if (!templateId) {
    navigate("/overview");
  }

  const [deployStack] = useMutation(DEPLOY_STACK);

  const handleDeployStack = async () => {
    if (!templateId) {
      throw new Error("Template ID is required");
    }

    const { data: deployStackData } = await deployStack({
      variables: {
        name: "test",
        templateId,
        cloudAccountId: domain,
      },
    });

    const id = deployStackData?.deployStack.id;

    toast.success(`Stack deployed successfully with id: ${id}`);

    navigate("/overview");
  };

  return (
    <Page.Container>
      <Page.Header>
        <Page.Title>Deploy a new Stack</Page.Title>
        <Page.Description>
          Deploy a new stack on one of your domain
        </Page.Description>
      </Page.Header>
      <Page.Body className="h-full pt-16">
        <Stepper.Root leftBorder shouldAnimate>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <TfiPackage className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Choose the cloud account</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="flex flex-col space-y-1">
                    <div className="font-medium text-sm">
                      All of your clouds
                    </div>
                    <p className="text-gray-500 text-sm">
                      Choose on which cloud / account you want the stack to be
                      deployed on.
                    </p>
                    <div className="pt-4">
                      <Select
                        disabled={loading}
                        onValueChange={(value) => {
                          setDomain(value);
                        }}
                        value={domain}
                      >
                        <SelectTrigger className="w-[300px]">
                          <SelectValue placeholder="Select a cloud account" />
                        </SelectTrigger>
                        <SelectContent>
                          {!loading &&
                            data?.me.organization.cloudAccounts.map((cloud) => (
                              <SelectItem key={cloud.id} value={cloud.id}>
                                {cloud.name}
                              </SelectItem>
                            ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="flex flex-col gap-2 pt-4">
                      <Label htmlFor="name">Name of the stack</Label>
                      <Input
                        className="w-[300px]"
                        onChange={(e) => setName(e.target.value)}
                        placeholder="Name of the stack"
                        value={name}
                      />
                    </div>
                    <div className="pt-4">
                      <Button
                        className="w-44"
                        disabled={!(name && domain)}
                        onClick={() => setStep(1)}
                      >
                        Next step
                      </Button>
                    </div>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <MdPreview className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Review the changes</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="font-medium text-sm">Plan of action</div>
                  <p className="text-gray-500 text-sm">
                    All of these ressources will be created on the domain you
                    selected.
                  </p>
                  <div className="flex flex-col gap-4 pt-4">
                    <div className="flex items-center gap-3">
                      <Checkbox
                        checked={agreement}
                        id="terms"
                        onCheckedChange={(v) => {
                          if (v === "indeterminate") {
                            setAgreement(false);
                          } else {
                            setAgreement(v);
                          }
                        }}
                      />
                      <Label htmlFor="terms">I have review the stack</Label>
                    </div>
                    <Button
                      className="w-44"
                      onClick={() => {
                        setPlanReview(true);
                        setStep(2);
                      }}
                    >
                      Next step
                    </Button>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item className="h-full">
            <Stepper.Header>
              <Stepper.Icon>
                <MdChecklist className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Agreement</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="font-medium text-sm">
                    Change will be applied
                  </div>
                  <p className="text-gray-500 text-sm">
                    You agree to the term and conditions of the service. You
                    take responsibility for the ressources you are deploying.
                  </p>
                  <div className="pt-4">
                    <div className="flex items-center gap-3">
                      <Checkbox
                        checked={applyStack}
                        id="terms"
                        onCheckedChange={(v) => {
                          if (v === "indeterminate") {
                            setApplyStack(false);
                          } else {
                            setApplyStack(v);
                          }
                        }}
                      />
                      <Label htmlFor="terms">
                        Accept the changes and the risks of deploying the stack
                      </Label>
                    </div>
                    <div className="pt-8">
                      <Button
                        disabled={!agreement}
                        onClick={() => {
                          // setStep(3);
                          handleDeployStack();
                        }}
                      >
                        Deploy the SPA stack
                      </Button>
                    </div>
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
