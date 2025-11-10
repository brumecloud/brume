import { useFragment } from "@apollo/client";
import { icons, Pickaxe } from "lucide-react";
import { useState } from "react";
import { useParams } from "react-router-dom";
import { Label } from "src/components/ui/label";
import { RadioGroup, RadioGroupItem } from "src/components/ui/radio-group";
import { BrumeForm } from "@/components/form/form";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { BUILDER_FRAGMENT } from "@/gql/builder.graphql";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

export const BuildTypeValues = {
  GenericImage: "generic-image",
  Dockerfile: "dockerfile",
  StaticWebsite: "static-website",
} as const;

export type BuildType = (typeof BuildTypeValues)[keyof typeof BuildTypeValues];

export const BuilderPage = () => {
  const { serviceId } = useParams<RouteParams>();

  const { data: builder, complete } = useFragment({
    from: `Builder:${serviceId}`,
    fragment: BUILDER_FRAGMENT,
    fragmentName: "BuilderFragment",
  });

  if (!complete) {
    throw new Error("Builder not complete");
  }

  const [_buildType, setBuildType] = useState<BuildType>("static-website");
  const { ui: uiSchema, ...schema } = JSON.parse(builder.schema);

  const steps = uiSchema["brume:stepper"] as string[];

  return (
    <Page.Container>
      <Page.Header>
        <Page.Title>Building the service</Page.Title>
        <Page.Description>Configure how your service is built</Page.Description>
      </Page.Header>
      <Page.Body className="pt-8">
        <Stepper.Root leftBorder>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <Pickaxe className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Builder selection</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {() => (
                <>
                  <div className="flex flex-col space-y-1">
                    <div className="font-medium text-sm">Select a builder</div>
                    <p className="text-gray-500 text-sm">
                      Select the builder you want to use for this service (you
                      can always add more builder using the marketplace).
                    </p>
                  </div>
                  <RadioGroup
                    className="flex flex-col space-y-2"
                    defaultValue="single-page-application-builder"
                    onValueChange={(value) => setBuildType(value as BuildType)}
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
                        <span className="font-medium">
                          Single Page Application Builder
                        </span>
                        <p className="text-gray-500 text-sm">
                          Simple builder using Vercel deploy API to build your
                          single page application.
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
