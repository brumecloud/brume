import { GET_MACHINES } from "@/gql/machine.graphql";
import {
  MachineSchema,
  type Machine,
} from "@/schemas/machine.schema";
import { useQuery } from "@apollo/client";
import { z } from "zod";

export const useMachine = (): {
  machines?: Machine[];
  loading: boolean;
  error?: Error;
} => {
  const { data, loading } = useQuery(GET_MACHINES, {
    fetchPolicy: "cache-first",
  });

  if (loading || !data) {
    return {
      loading: true,
    };
  } else {
    const rawData = z.array(MachineSchema).safeParse(data?.machine);

    if (!rawData.success) {
      throw new Error(rawData.error.message);
    } else {
      return {
        machines: rawData.data,
        loading: false,
      };
    }
  }
};
