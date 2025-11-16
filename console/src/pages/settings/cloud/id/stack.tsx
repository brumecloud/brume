import { useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { GET_CLOUD_ACCOUNT } from "@/gql/cloud.graphql";
import type { RouteParams } from "@/router/router.param";

export const CloudStackPage = () => {
  const { cloudAccountId } = useParams<typeof RouteParams>() as {
    cloudAccountId: string;
  };

  const { data, loading } = useQuery(GET_CLOUD_ACCOUNT, {
    variables: {
      id: cloudAccountId,
    },
    fetchPolicy: "cache-and-network",
  });

  if (loading || !data) {
    return null;
  }

  return (
    <div>
      <h1>Overview</h1>
    </div>
  );
};
