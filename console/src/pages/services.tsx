import { gql } from "@/_apollo";
import { Button } from "@/components/ui/button";
import { PROJECT_FRAGMENT } from "@/gql/project.graphql";
import type { RouteParams } from "@/router/router";
import { modalState } from "@/state/modal.state";
import { useFragment } from "@apollo/client";
import { useParams } from "react-router-dom";
import { useSnapshot } from "valtio";

export const ServiceFragment = gql(`
  fragment ServiceFragment on Service {
      name
      id
      deployments {
        ...DeploymentFragment @unmask
      }

      liveBuilder {
        ...BuilderFragment @unmask
      }
      draftBuilder {
        ...BuilderFragment @unmask
      }
      liveRunner {
        ...RunnerFragment @unmask
      }
      draftRunner {
        ...RunnerFragment @unmask
      }
   }
`);

export const ServicePage = () => {
  const snap = useSnapshot(modalState);
  const { projectId } = useParams<RouteParams>();

  if (!projectId) {
    throw new Error("No project ID found in the URL");
  }

  const { data } = useFragment({
    from: `Project:${projectId}`,
    fragment: PROJECT_FRAGMENT,
  });

  if (!data) {
    throw new Error("No data for the current project ?");
  }

  return (
    <div>
      Project {data.name}'s services
      <Button onClick={() => snap.setCreateServiceModalOpen(true)}>
        Add a service
      </Button>
    </div>
  );
};
