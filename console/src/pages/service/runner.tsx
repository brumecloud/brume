import { useFragment } from "@apollo/client";
import { icons, Pickaxe } from "lucide-react";
import { useParams } from "react-router-dom";
import { BrumeForm } from "@/components/form/form";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { RUNNER_FRAGMENT } from "@/gql/runner.graphql";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

export const RunnerPage = () => {
  const { serviceId } = useParams<RouteParams>();

  const { data: runner, complete } = useFragment({
    from: `Runner:${serviceId}`,
    fragment: RUNNER_FRAGMENT,
    fragmentName: "RunnerFragment",
  });

  if (!complete) {
    throw new Error("Service not complete");
  }

  if (!complete) {
    throw new Error("Builder not complete");
  }

  const { ui: uiSchema, ...schema } = JSON.parse(runner.schema);

  const steps = uiSchema["brume:stepper"] as string[];

  return (
    <Page.Container>
      <Page.Body className="pt-4">
        <Stepper.Root leftBorder>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <Pickaxe className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Runner selection</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {() => (
                <>
                  <div className="flex flex-col space-y-1">
                    <div className="font-medium text-sm">Select a runner</div>
                    <p className="text-gray-500 text-sm">
                      Select the right runner for your service.
                    </p>
                  </div>
                  <RadioGroup
                    className="flex flex-col space-y-2"
                    defaultValue="single-page-application-builder"
                  >
                    <div className="flex space-x-2">
                      <RadioGroupItem
                        defaultChecked
                        value="single-page-application-builder"
                      />
                      <Label
                        className="flex flex-col space-y-1"
                        htmlFor="static-website"
                      >
                        <span className="font-medium">Cloudfront SPA</span>
                        <p className="text-gray-500 text-sm">
                          Deploy your single page application to Cloudfront.
                        </p>
                      </Label>
                    </div>
                  </RadioGroup>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          {steps.map((step, index) => {
            const stepSchema = schema[step];
            if (!stepSchema) {
              return null;
            }

            const title = stepSchema.title as string;
            let Icon = <Pickaxe />;
            if (stepSchema.icon && stepSchema.icon !== "") {
              // @ts-expect-error
              Icon = icons[stepSchema.icon];
            }

            return (
              <Stepper.Item
                className={cn(
                  index === steps.length - 1 && "h-full min-h-[500px]"
                )}
                key={step}
              >
                <Stepper.Header>
                  <Stepper.Icon>
                    {/* @ts-expect-error */}
                    <Icon className="h-5 w-5" />
                  </Stepper.Icon>
                  <Stepper.Title>{title}</Stepper.Title>
                </Stepper.Header>
                <Stepper.Body>
                  {() => (
                    <div>
                      <BrumeForm schema={stepSchema} uiSchema={uiSchema} />
                    </div>
                  )}
                </Stepper.Body>
              </Stepper.Item>
            );
          })}
        </Stepper.Root>
      </Page.Body>
    </Page.Container>
  );
};
