import { describe, expect, it } from "vite-plus/test";

import { DecodeError } from "./decode";
import { parseServices } from "./services";

describe("parseServices", () => {
  const raw = {
    id: "s-1",
    environmentId: "e-1",
    name: "brume-api",
    slug: "brume-api",
    kind: "github",
    status: "online",
  };

  it("decodes the API response", () => {
    expect(parseServices([raw])).toEqual([raw]);
  });

  it("rejects an unknown kind or status", () => {
    expect(() => parseServices([{ ...raw, kind: "lambda" }])).toThrow(DecodeError);
    expect(() => parseServices([{ ...raw, status: "sleeping" }])).toThrow(DecodeError);
  });
});
