
import { MachineSchema, type Machine } from "@/schemas/machine.schema";
import { z } from "zod";

export const useMachine = (): {
  machines?: Machine[];
  loading: boolean;
  error?: Error;
} => {
  if (loading || !data) {
    return {
      loading: true,
    };
  }
    if (error) {
      console.error(error);
      throw new Error(error.message);
    }

    const rawData = z.array(MachineSchema).safeParse(data?.machine);

    if (rawData.success) {
      return {
        machines: rawData.data,
        loading: false,
      };
    } 
      throw new Error(rawData.error.message);
};
