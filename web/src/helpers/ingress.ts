import type { Ingress } from "~/api/types";

import { decodeRecord, readString } from "./decode";

export function parseIngress(raw: unknown): Ingress | null {
  if (raw === null) {
    return null;
  }
  return { zone: readString(decodeRecord(raw, "ingress"), "zone", "ingress") };
}

// The subdomain of the zone that the active deployment serves, as `demo.example.com`.
export function publicHost(domains: readonly string[], zone: string): string | undefined {
  return domains.find((domain) => domain.endsWith(`.${zone}`));
}
