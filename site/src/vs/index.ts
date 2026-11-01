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

export function competitor(slug: VsSlug): Competitor {
  const found = competitors.find((candidate) => candidate.slug === slug);
  if (!found) throw new Error(`No comparison for ${slug}`);
  return found;
}
