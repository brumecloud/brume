import { Globe, KeyRound, LockKeyhole } from "lucide-react";
import type { AuthMode } from "./api";

export const ACCESS_COPY: Record<
  AuthMode,
  { icon: typeof Globe; label: string; description: string }
> = {
  none: {
    icon: Globe,
    label: "Public",
    description: "Anyone with the link can view",
  },
  password: {
    icon: KeyRound,
    label: "Password protected",
    description: "Visitors must enter the password",
  },
  token: {
    icon: LockKeyhole,
    label: "Private",
    description: "Only invited recipients can view",
  },
};

/** Selector order matches the old management page: most restrictive first. */
export const ACCESS_MODES: AuthMode[] = ["token", "password", "none"];
