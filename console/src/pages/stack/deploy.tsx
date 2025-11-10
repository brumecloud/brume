import { useState } from "react";
import { MdChecklist, MdPreview } from "react-icons/md";
import { TfiPackage } from "react-icons/tfi";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export const DeployStack = () => {
  const [domain, setDomain] = useState("");
  const [agreement, setAgreement] = useState(false);
  const [_planReview, setPlanReview] = useState(false);
  const [applyStack, setApplyStack] = useState(false);

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
                        onValueChange={(value) => {
                          setStep(1);
                          setDomain(value);
                        }}
                        value={domain}
                      >
                        <SelectTrigger className="w-[300px]">
                          <SelectValue placeholder="Select a cloud account" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="dev">Dev AWS</SelectItem>
                          <SelectItem value="prod">Production AWS</SelectItem>
                        </SelectContent>
                      </Select>
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
                          setStep(3);
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
