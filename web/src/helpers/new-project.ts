import type { CreatedProject, ProjectId, Repository } from "~/api/types";

import { decodeArray, decodeRecord, readString } from "./decode";

const MAX_IMAGE_LENGTH = 255;
// [registry[:port]/][namespace/…]name[:tag][@sha256:digest], lowercase like Docker requires.
const IMAGE_REFERENCE =
  /^(?:[a-z0-9.-]+(?::\d+)?\/)?(?:[a-z0-9]+(?:[._-][a-z0-9]+)*\/)*[a-z0-9]+(?:[._-][a-z0-9]+)*(?::[A-Za-z0-9_][A-Za-z0-9_.-]{0,127})?(?:@sha256:[a-f0-9]{64})?$/;

export type NewProjectStep = "github" | "static" | "database" | "image";

export interface NewProjectSearch {
  readonly step?: NewProjectStep;
}

export function parseNewProjectSearch(raw: Record<string, unknown>): NewProjectSearch {
  const step = raw.step;
  return step === "github" || step === "static" || step === "database" || step === "image"
    ? { step }
    : {};
}

export function parseImageReference(raw: string): string | undefined {
  const reference = raw.trim();
  return reference.length <= MAX_IMAGE_LENGTH && IMAGE_REFERENCE.test(reference)
    ? reference
    : undefined;
}

export function parseRepositories(raw: unknown): Repository[] {
  return decodeArray(
    raw,
    (item) => ({
      fullName: readString(decodeRecord(item, "repository"), "fullName", "repository"),
    }),
    "repositories",
  );
}

export function parseCreatedProject(raw: unknown): CreatedProject {
  const record = decodeRecord(raw, "created project");
  return {
    projectId: readString(record, "projectId", "created project") as ProjectId,
    environmentName: readString(record, "environmentName", "created project"),
  };
}
