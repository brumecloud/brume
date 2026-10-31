import { DecodeError, decodeArray } from "./decode";

// An owner, then a repository name: only characters that are safe in a URL path, no `..`.
const REPOSITORY = /^[A-Za-z0-9-]{1,39}\/(?!\.\.?$)[A-Za-z0-9_.-]{1,100}$/;

// The repository as a path segment of the API, or undefined if the name is not a GitHub one.
export function repositoryPath(fullName: string): string | undefined {
  return REPOSITORY.test(fullName) ? fullName : undefined;
}

export function parseBranches(raw: unknown): string[] {
  return decodeArray(
    raw,
    (item) => {
      if (typeof item !== "string") {
        throw new DecodeError("branch: expected a string");
      }
      return item;
    },
    "branches",
  );
}
