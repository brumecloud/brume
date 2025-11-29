import { useFragment } from "@apollo/client";
import { useParams } from "react-router-dom";
import { STACK_FRAGMENT } from "@/gql/stack.graphql";

export const StackOverview = () => {
  const { stackId } = useParams();

  const { data, complete } = useFragment({
    fragment: STACK_FRAGMENT,
    from: `Stack:${stackId}`,
  });

  if (!(complete && data)) {
    return null;
  }

  return (
    <div className="flex flex-col">
      <div className="px-32 pt-8">
        <div className="flex flex-row gap-4" />
      </div>
    </div>
  );
};
