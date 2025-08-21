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
import { useState } from "react";
import { MdChecklist, MdPreview } from "react-icons/md";
import { TfiPackage } from "react-icons/tfi";

export const DeployStack = () => {
  const [domain, setDomain] = useState("");
  const [agreement, setAgreement] = useState(false);
  const [planReview, setPlanReview] = useState(false);

  return (
    <Page.Container>
      <Page.Header>
        <Page.Title>New Deploy Stack</Page.Title>
        <Page.Description>
          Deploy a new stack on one of your domain
        </Page.Description>
      </Page.Header>
      <Page.Body className="h-full pt-16">
        <Stepper.Root shouldAnimate>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <TfiPackage className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Choose the domain</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="flex flex-col space-y-1">
                    <div className="text-sm font-medium">
                      All of your domains
                    </div>
                    <p className="text-sm text-gray-500">
                      Choose on which domain / account you want the
                      stack to be deployed on.
                    </p>
                    <div className="pt-4">
                      <Select
                        onValueChange={(value) => {
                          setStep(1);
                          setDomain(value);
                        }}>
                        <SelectTrigger className="w-[300px]">
                          <SelectValue placeholder="Select a domain" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="dev">Dev AWS</SelectItem>
                          <SelectItem value="prod">
                            Production AWS
                          </SelectItem>
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
                  <div className="text-sm font-medium">
                    Plan of action
                  </div>
                  <p className="text-sm text-gray-500">
                    All of these ressources will be created on the
                    domain you selected.
                  </p>
                  <div className="pt-4">
                    <Button
                      onClick={() => {
                        setPlanReview(true);
                        setStep(2);
                      }}>
                      I reviewed the plan
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
                  <div className="text-sm font-medium">
                    Change will be applied
                  </div>
                  <p className="text-sm text-gray-500">
                    You agree to the term and conditions of the
                    service. You take responsibility for the
                    ressources you are deploying.
                  </p>
                  <div className="pt-4">
                    <div className="flex items-center gap-3">
                      <Checkbox
                        id="terms"
                        checked={agreement}
                        onCheckedChange={(v) => {
                          if (v == "indeterminate") {
                            setAgreement(false);
                          } else {
                            setAgreement(v);
                          }
                        }}
                      />
                      <Label htmlFor="terms">
                        Accept the changes and the risks of deploying
                        the stack
                      </Label>
                    </div>
                    <div className="pt-8">
                      <Button
                        onClick={() => {
                          setStep(3);
                        }}
                        disabled={!agreement}>
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
