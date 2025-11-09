import { useFragment } from "@apollo/client";
import { zodResolver } from "@hookform/resolvers/zod";
import { SquareTerminal } from "lucide-react";
import { useForm } from "react-hook-form";
import { useBlocker, useParams } from "react-router-dom";
import { toast } from "sonner";
import { z } from "zod";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { FormField } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { SourceFragment } from "@/gql/source.graphql";
import type { RouteParams } from "@/router/router.param";
import { cn } from "@/utils";

const GitSourceSchema = z.object({
  repository: z.string(),
  branch: z.string(),
  provider: z.enum(["github", "gitlab"]),
});

export const SourcePage = () => {
  const { serviceId } = useParams<RouteParams>();

  const { data: source, complete } = useFragment({
    from: `Source:${serviceId}`,
    fragment: SourceFragment,
    fragmentName: "SourceFragment",
  });

  if (!complete) {
    throw new Error("Source not complete");
  }

  if (source?.type !== "git") {
    throw new Error("Source is not a git repository");
  }

  const sourceData = GitSourceSchema.safeParse(source?.data);

  if (!sourceData.success) {
    throw new Error("Source data is invalid");
  }

  const form = useForm<{ name: string; branch: string }>({
    resolver: zodResolver(
      z.object({
        name: z.string(),
        branch: z.string(),
      })
    ),
    mode: "onChange",
    defaultValues: {
      name: sourceData.data.repository,
      branch: sourceData.data.branch,
    },
  });

  const blocker = useBlocker(() => form.formState.isDirty);

  if (blocker.state === "blocked") {
    toast.warning("You have unsaved changes");
  }

  return (
    <Page.Container>
      <Page.Header>
        <Page.Title>Sourcing the code</Page.Title>
        <Page.Description>
          Configure the source of code of the service
        </Page.Description>
      </Page.Header>
      <Page.Body className="pt-8">
        <Stepper.Root leftBorder>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <SquareTerminal className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Git repository</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {() => (
                <>
                  <p className="text-sm">Define the name of the service</p>
                  <div>
                    <Label className="text-xs" htmlFor="image">
                      Repository URL
                    </Label>
                    <FormField
                      control={form.control}
                      name="name"
                      render={({ field, fieldState }) => (
                        <Input
                          {...field}
                          className={cn(
                            "mt-1 w-[400px]",
                            fieldState.isDirty && "border-green-500"
                          )}
                          placeholder="github.com/brumecloud/brume"
                        />
                      )}
                    />
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item className="h-full">
            <Stepper.Header>
              <Stepper.Icon>
                <SquareTerminal className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Trigger configuration</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {() => (
                <>
                  <p className="text-sm">
                    Each commit push to this branch will trigger a build of the
                    service.
                  </p>
                  <div>
                    <Label className="text-xs" htmlFor="image">
                      Branch name
                    </Label>
                    <FormField
                      control={form.control}
                      name="branch"
                      render={({ field, fieldState }) => (
                        <Input
                          {...field}
                          className={cn(
                            "mt-1 w-[400px]",
                            fieldState.isDirty && "border-green-500"
                          )}
                          placeholder="main"
                        />
                      )}
                    />
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
