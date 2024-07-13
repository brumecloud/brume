import { ME_QUERY } from "@/gql/user.graphql";
import { UserSchema, type User } from "@/schemas/user.schema";
import { useQuery } from "@apollo/client";
import { toast } from "sonner";

export const useUser = (): {
  me: User | null;
  loading: boolean;
  error?: Error | null;
} => {
  const { data, loading } = useQuery(ME_QUERY, {
    fetchPolicy: "cache-and-network",
  });

  if (loading) {
    return {
      me: null,
      loading: true,
    };
  } else {
    const parsedData = UserSchema.safeParse(data.me);

    if (!parsedData.success) {
      toast.error("Failed to parse user data");
      throw new Error("Failed to parse user data");
    }

    return {
      me: parsedData.data,
      loading: false,
    };
  }
};
