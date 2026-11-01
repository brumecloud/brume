import { parseEnvironmentName } from "./projects";

// The server's `Slug` rules: the name is also the environment's URL segment and Docker network.
export function environmentNameError(name: string, taken: readonly string[]): string | null {
  if (!parseEnvironmentName(name)) {
    return "Lowercase letters, digits and hyphens, starting and ending with a letter or a digit, 63 at most.";
  }
  return taken.includes(name) ? `An environment named ${name} already exists.` : null;
}
