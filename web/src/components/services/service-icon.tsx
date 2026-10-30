import { GlobeIcon } from "lucide-react";

import type { ServiceKind } from "~/api/types";
import {
  DockerIcon,
  GitHubIcon,
  MongoIcon,
  PostgresIcon,
  RedisIcon,
} from "~/components/icons/brand-icons";

export function ServiceIcon({ kind }: { readonly kind: ServiceKind }) {
  switch (kind) {
    case "github":
      return <GitHubIcon className="size-5" />;
    case "static":
      return <GlobeIcon className="size-5" />;
    case "image":
      return <DockerIcon className="size-5 text-brand-docker" />;
    case "postgres":
      return <PostgresIcon className="size-5 text-brand-postgres" />;
    case "mongodb":
      return <MongoIcon className="size-5 text-brand-mongodb" />;
    case "redis":
      return <RedisIcon className="size-5 text-brand-redis" />;
  }
}

export function ServiceIconTile({
  kind,
  name,
}: {
  readonly kind: ServiceKind;
  readonly name: string;
}) {
  return (
    <span
      title={name}
      className="flex size-10 items-center justify-center rounded-lg border bg-card shadow-xs"
    >
      <ServiceIcon kind={kind} />
    </span>
  );
}
