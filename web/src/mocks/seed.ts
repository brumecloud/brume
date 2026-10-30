import type { Environment, EnvironmentId, Project, ProjectId } from "~/api/types";

const project = (id: string, name: string, description: string, updatedAt: string): Project => ({
  id: id as ProjectId,
  name,
  description,
  updatedAt,
});

const environment = (projectId: ProjectId, name: string): Environment => ({
  id: `${projectId}-${name}` as EnvironmentId,
  projectId,
  name,
});

export const brume = project(
  "c0a1d9f0-fd9b-4683-b608-2bf60613b499",
  "brume",
  "API, worker, MongoDB et Redis.",
  "2026-10-29T21:12:00Z",
);
const beaubon = project(
  "5b0e7c52-8d0b-4a3e-9a51-3c1f0e8e2d10",
  "beaubon",
  "Beaubon : API Go, app Expo, site web.",
  "2026-10-28T09:40:00Z",
);
const melled = project(
  "9f3c2a71-64b5-4f0e-b2d8-0a7e5c1d4b22",
  "melled",
  "",
  "2026-10-21T16:05:00Z",
);
const eve = project("2d7e9b14-0c3a-4e61-8f5d-6b2a1c9e7f33", "eve", "", "2026-10-12T11:30:00Z");

export const projects: readonly Project[] = [brume, beaubon, melled, eve];

export const environments: readonly Environment[] = [
  environment(brume.id, "production"),
  environment(brume.id, "staging"),
  environment(beaubon.id, "production"),
  environment(melled.id, "production"),
  environment(eve.id, "dev"),
];
