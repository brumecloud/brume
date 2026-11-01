// Every comparison, self-hosted platforms first, in the order of the /vs page.

import { coolify } from "./coolify";
import { dokploy } from "./dokploy";
import { caprover } from "./caprover";
import { portainer } from "./portainer";
import { kamal } from "./kamal";
import { dokku } from "./dokku";
import { easypanel } from "./easypanel";
import { railway } from "./railway";
import { render } from "./render";
import { fly } from "./fly";
import { vercel } from "./vercel";
import { heroku } from "./heroku";
import { managedRows, selfHostedRows } from "./brume";
import type { Competitor, VsSlug } from "./model";

export const competitors: readonly Competitor[] = [
  coolify,
  dokploy,
  caprover,
  portainer,
  kamal,
  dokku,
  easypanel,
  railway,
  render,
  fly,
  vercel,
  heroku,
];

/** The comparison for a slug read from a URL, if there is one. */
export function findCompetitor(slug: string): Competitor | undefined {
  return competitors.find((candidate) => candidate.slug === slug);
}

export function competitor(slug: VsSlug): Competitor {
  const found = findCompetitor(slug);
  if (!found) throw new Error(`No comparison for ${slug}`);
  return found;
}

// Every table lists its rows in the order of its family in brume.ts, so the pages read alike.
// A row out of place fails the build.
for (const entry of competitors) {
  const order = entry.kind === "self-hosted" ? selfHostedRows : managedRows;
  const positions = entry.rows.map((row) => order.indexOf(row.key));
  const inOrder = positions.every(
    (position, index) => position >= 0 && (index === 0 || position > (positions[index - 1] ?? -1)),
  );
  if (!inOrder) throw new Error(`The rows of ${entry.slug} are not in the order of brume.ts`);
}
